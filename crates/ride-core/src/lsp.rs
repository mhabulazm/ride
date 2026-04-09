use crate::settings::LspServerConfig;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;

// --- LSP types ---

#[derive(Debug, Clone, Serialize)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LspPosition {
    pub line: u32,
    pub character: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LspRange {
    pub start: LspPosition,
    pub end: LspPosition,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LspLocation {
    pub uri: String,
    #[serde(rename = "range")]
    pub range: LspRange,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LspDiagnostic {
    pub range: LspRange,
    pub severity: Option<u32>,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub file: PathBuf,
    pub line: usize,
    pub col: usize,
    pub end_col: usize,
    pub severity: DiagnosticSeverity,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

#[derive(Debug, Clone)]
pub struct CompletionItem {
    pub label: String,
    pub detail: Option<String>,
    pub insert_text: Option<String>,
    pub kind: CompletionKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionKind {
    Text,
    Method,
    Function,
    Constructor,
    Field,
    Variable,
    Class,
    Interface,
    Module,
    Property,
    Keyword,
    Snippet,
    Other,
}

#[derive(Debug, Clone)]
pub struct TextEdit {
    pub range: LspRange,
    pub new_text: String,
}

#[derive(Debug, Clone)]
pub struct WorkspaceEdit {
    pub changes: HashMap<PathBuf, Vec<TextEdit>>,
}

#[derive(Debug, Clone)]
pub struct CodeAction {
    pub title: String,
    pub kind: Option<String>,
    pub edit: Option<WorkspaceEdit>,
}

#[derive(Debug, Clone)]
pub struct ReferenceLocation {
    pub file: PathBuf,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub enum LspEvent {
    Diagnostics {
        file: PathBuf,
        diagnostics: Vec<Diagnostic>,
    },
    HoverResult {
        contents: String,
    },
    GotoDefinition {
        file: PathBuf,
        line: usize,
        col: usize,
    },
    Completion {
        items: Vec<CompletionItem>,
    },
    CodeActions {
        actions: Vec<CodeAction>,
    },
    References {
        locations: Vec<ReferenceLocation>,
    },
    FormatResult {
        edits: Vec<TextEdit>,
    },
    Error(String),
}

// --- JSON-RPC ---

fn encode_message(content: &str) -> Vec<u8> {
    format!("Content-Length: {}\r\n\r\n{}", content.len(), content).into_bytes()
}

fn decode_message<R: BufRead>(reader: &mut R) -> Option<String> {
    let mut header = String::new();
    let mut content_length: usize = 0;

    loop {
        header.clear();
        if reader.read_line(&mut header).ok()? == 0 {
            return None;
        }
        let trimmed = header.trim();
        if trimmed.is_empty() {
            break;
        }
        if let Some(len_str) = trimmed.strip_prefix("Content-Length: ") {
            content_length = len_str.parse().ok()?;
        }
    }

    if content_length == 0 {
        return None;
    }

    let mut body = vec![0u8; content_length];
    reader.read_exact(&mut body).ok()?;
    String::from_utf8(body).ok()
}

// --- LSP Client ---

pub struct LspClient {
    process: Child,
    writer: std::io::BufWriter<std::process::ChildStdin>,
    events_rx: Receiver<LspEvent>,
    next_id: i64,
    root_uri: String,
    pending_requests: Arc<Mutex<HashMap<i64, String>>>, // id -> method
}

impl LspClient {
    pub fn start(config: &LspServerConfig, root_path: &Path) -> Option<Self> {
        let mut child = Command::new(&config.command)
            .args(&config.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .ok()?;

        let stdin = child.stdin.take()?;
        let stdout = child.stdout.take()?;
        let writer = std::io::BufWriter::new(stdin);

        let (tx, rx) = mpsc::channel();
        let pending_requests = Arc::new(Mutex::new(HashMap::<i64, String>::new()));
        let pending_clone = Arc::clone(&pending_requests);

        // Reader thread
        thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            while let Some(msg) = decode_message(&mut reader) {
                let parsed: Value = match serde_json::from_str(&msg) {
                    Ok(v) => v,
                    Err(_) => continue,
                };

                // Handle notifications
                if let Some(method) = parsed.get("method").and_then(|m| m.as_str()) {
                    if method == "textDocument/publishDiagnostics" {
                        if let Some(params) = parsed.get("params") {
                            let uri = params.get("uri").and_then(|u| u.as_str()).unwrap_or("");
                            let file = uri_to_path(uri);

                            let lsp_diags: Vec<LspDiagnostic> = params
                                .get("diagnostics")
                                .and_then(|d| serde_json::from_value(d.clone()).ok())
                                .unwrap_or_default();

                            let diagnostics: Vec<Diagnostic> = lsp_diags
                                .into_iter()
                                .map(|d| Diagnostic {
                                    file: file.clone(),
                                    line: d.range.start.line as usize,
                                    col: d.range.start.character as usize,
                                    end_col: d.range.end.character as usize,
                                    severity: match d.severity {
                                        Some(1) => DiagnosticSeverity::Error,
                                        Some(2) => DiagnosticSeverity::Warning,
                                        Some(3) => DiagnosticSeverity::Info,
                                        _ => DiagnosticSeverity::Hint,
                                    },
                                    message: d.message,
                                })
                                .collect();

                            let _ = tx.send(LspEvent::Diagnostics { file, diagnostics });
                        }
                    }
                    continue;
                }

                // Handle responses
                if let Some(id) = parsed.get("id").and_then(|i| i.as_i64()) {
                    let method = pending_clone
                        .lock()
                        .unwrap()
                        .remove(&id)
                        .unwrap_or_default();
                    if let Some(result) = parsed.get("result") {
                        let _ = tx.send(parse_response(&method, result));
                    } else if let Some(error) = parsed.get("error") {
                        let msg = error
                            .get("message")
                            .and_then(|m| m.as_str())
                            .unwrap_or("Unknown LSP error");
                        let _ = tx.send(LspEvent::Error(msg.to_string()));
                    }
                }
            }
        });

        let root_uri = path_to_uri(root_path);

        let mut client = Self {
            process: child,
            writer,
            events_rx: rx,
            next_id: 1,
            root_uri,
            pending_requests,
        };

        client.send_initialize();
        Some(client)
    }

    fn send_request(&mut self, method: &str, params: Value) -> i64 {
        let id = self.next_id;
        self.next_id += 1;

        let msg = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });

        self.pending_requests.lock().unwrap().insert(id, method.to_string());
        let content = msg.to_string();
        let encoded = encode_message(&content);
        let _ = self.writer.write_all(&encoded);
        let _ = self.writer.flush();
        id
    }

    fn send_notification(&mut self, method: &str, params: Value) {
        let msg = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        });

        let content = msg.to_string();
        let encoded = encode_message(&content);
        let _ = self.writer.write_all(&encoded);
        let _ = self.writer.flush();
    }

    fn send_initialize(&mut self) {
        let params = json!({
            "processId": std::process::id(),
            "rootUri": self.root_uri,
            "capabilities": {
                "textDocument": {
                    "hover": { "contentFormat": ["plaintext"] },
                    "definition": {},
                    "publishDiagnostics": {},
                    "completion": {
                        "completionItem": {
                            "snippetSupport": false,
                            "insertReplaceSupport": false
                        }
                    },
                    "codeAction": {
                        "codeActionLiteralSupport": {
                            "codeActionKind": {
                                "valueSet": ["quickfix", "refactor", "source"]
                            }
                        }
                    },
                    "references": {},
                    "formatting": { "dynamicRegistration": false }
                }
            }
        });
        self.send_request("initialize", params);
    }

    pub fn initialized(&mut self) {
        self.send_notification("initialized", json!({}));
    }

    pub fn did_open(&mut self, path: &Path, language_id: &str, text: &str) {
        let uri = path_to_uri(path);
        self.send_notification(
            "textDocument/didOpen",
            json!({
                "textDocument": {
                    "uri": uri,
                    "languageId": language_id,
                    "version": 1,
                    "text": text,
                }
            }),
        );
    }

    pub fn did_change(&mut self, path: &Path, version: i32, text: &str) {
        let uri = path_to_uri(path);
        self.send_notification(
            "textDocument/didChange",
            json!({
                "textDocument": { "uri": uri, "version": version },
                "contentChanges": [{ "text": text }],
            }),
        );
    }

    pub fn did_save(&mut self, path: &Path) {
        let uri = path_to_uri(path);
        self.send_notification(
            "textDocument/didSave",
            json!({
                "textDocument": { "uri": uri }
            }),
        );
    }

    pub fn hover(&mut self, path: &Path, line: u32, character: u32) {
        let uri = path_to_uri(path);
        self.send_request(
            "textDocument/hover",
            json!({
                "textDocument": { "uri": uri },
                "position": { "line": line, "character": character },
            }),
        );
    }

    pub fn goto_definition(&mut self, path: &Path, line: u32, character: u32) {
        let uri = path_to_uri(path);
        self.send_request(
            "textDocument/definition",
            json!({
                "textDocument": { "uri": uri },
                "position": { "line": line, "character": character },
            }),
        );
    }

    pub fn completion(&mut self, path: &Path, line: u32, character: u32) {
        let uri = path_to_uri(path);
        self.send_request(
            "textDocument/completion",
            json!({
                "textDocument": { "uri": uri },
                "position": { "line": line, "character": character },
            }),
        );
    }

    pub fn code_action(&mut self, path: &Path, line: u32, character: u32, diagnostics_json: Value) {
        let uri = path_to_uri(path);
        self.send_request(
            "textDocument/codeAction",
            json!({
                "textDocument": { "uri": uri },
                "range": {
                    "start": { "line": line, "character": character },
                    "end": { "line": line, "character": character }
                },
                "context": {
                    "diagnostics": diagnostics_json
                }
            }),
        );
    }

    pub fn references(&mut self, path: &Path, line: u32, character: u32) {
        let uri = path_to_uri(path);
        self.send_request(
            "textDocument/references",
            json!({
                "textDocument": { "uri": uri },
                "position": { "line": line, "character": character },
                "context": { "includeDeclaration": true }
            }),
        );
    }

    pub fn format(&mut self, path: &Path) {
        let uri = path_to_uri(path);
        self.send_request(
            "textDocument/formatting",
            json!({
                "textDocument": { "uri": uri },
                "options": {
                    "tabSize": 4,
                    "insertSpaces": true
                }
            }),
        );
    }

    pub fn poll_events(&self) -> Vec<LspEvent> {
        let mut events = Vec::new();
        while let Ok(event) = self.events_rx.try_recv() {
            events.push(event);
        }
        events
    }

    pub fn shutdown(&mut self) {
        self.send_request("shutdown", json!(null));
        self.send_notification("exit", json!(null));
        let _ = self.process.wait();
    }
}

impl Drop for LspClient {
    fn drop(&mut self) {
        let _ = self.process.kill();
    }
}

// --- LSP Manager (manages multiple servers by extension) ---

pub struct LspManager {
    clients: HashMap<String, LspClient>, // extension -> client
    pub diagnostics: HashMap<PathBuf, Vec<Diagnostic>>,
    pub hover_info: Option<String>,
    pub pending_goto: Option<(PathBuf, usize, usize)>,
    pub pending_completions: Option<Vec<CompletionItem>>,
    pub pending_code_actions: Option<Vec<CodeAction>>,
    pub pending_references: Option<Vec<ReferenceLocation>>,
    pub pending_format: Option<Vec<TextEdit>>,
    configs: HashMap<String, LspServerConfig>,
    root_path: PathBuf,
    initialized_servers: HashMap<String, bool>,
}

impl LspManager {
    pub fn new(configs: HashMap<String, LspServerConfig>, root_path: &Path) -> Self {
        Self {
            clients: HashMap::new(),
            diagnostics: HashMap::new(),
            hover_info: None,
            pending_goto: None,
            pending_completions: None,
            pending_code_actions: None,
            pending_references: None,
            pending_format: None,
            configs,
            root_path: root_path.to_path_buf(),
            initialized_servers: HashMap::new(),
        }
    }

    pub fn ensure_server_for_extension(&mut self, ext: &str) {
        if self.clients.contains_key(ext) {
            return;
        }
        if let Some(config) = self.configs.get(ext).cloned() {
            if let Some(client) = LspClient::start(&config, &self.root_path) {
                self.clients.insert(ext.to_string(), client);
                self.initialized_servers.insert(ext.to_string(), false);
            }
        }
    }

    pub fn did_open(&mut self, path: &Path, text: &str) {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let ext = ext.to_lowercase();
            self.ensure_server_for_extension(&ext);
            let lang_id = extension_to_language_id(&ext);
            if let Some(client) = self.clients.get_mut(&ext) {
                client.did_open(path, lang_id, text);
            }
        }
    }

    pub fn did_change(&mut self, path: &Path, version: i32, text: &str) {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let ext = ext.to_lowercase();
            if let Some(client) = self.clients.get_mut(&ext) {
                client.did_change(path, version, text);
            }
        }
    }

    pub fn did_save(&mut self, path: &Path) {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let ext = ext.to_lowercase();
            if let Some(client) = self.clients.get_mut(&ext) {
                client.did_save(path);
            }
        }
    }

    pub fn request_hover(&mut self, path: &Path, line: u32, col: u32) {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let ext = ext.to_lowercase();
            if let Some(client) = self.clients.get_mut(&ext) {
                self.hover_info = None;
                client.hover(path, line, col);
            }
        }
    }

    pub fn request_goto_definition(&mut self, path: &Path, line: u32, col: u32) {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let ext = ext.to_lowercase();
            if let Some(client) = self.clients.get_mut(&ext) {
                self.pending_goto = None;
                client.goto_definition(path, line, col);
            }
        }
    }

    pub fn request_completion(&mut self, path: &Path, line: u32, col: u32) {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let ext = ext.to_lowercase();
            self.ensure_server_for_extension(&ext);
            if let Some(client) = self.clients.get_mut(&ext) {
                self.pending_completions = None;
                client.completion(path, line, col);
            }
        }
    }

    pub fn request_code_actions(&mut self, path: &Path, line: u32, col: u32, diagnostics_json: Value) {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let ext = ext.to_lowercase();
            self.ensure_server_for_extension(&ext);
            if let Some(client) = self.clients.get_mut(&ext) {
                self.pending_code_actions = None;
                client.code_action(path, line, col, diagnostics_json);
            }
        }
    }

    pub fn request_references(&mut self, path: &Path, line: u32, col: u32) {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let ext = ext.to_lowercase();
            self.ensure_server_for_extension(&ext);
            if let Some(client) = self.clients.get_mut(&ext) {
                self.pending_references = None;
                client.references(path, line, col);
            }
        }
    }

    pub fn request_format(&mut self, path: &Path) {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let ext = ext.to_lowercase();
            self.ensure_server_for_extension(&ext);
            if let Some(client) = self.clients.get_mut(&ext) {
                self.pending_format = None;
                client.format(path);
            }
        }
    }

    pub fn poll(&mut self) {
        let exts: Vec<String> = self.clients.keys().cloned().collect();
        for ext in exts {
            // Check if we need to send initialized notification
            if let Some(false) = self.initialized_servers.get(&ext) {
                if let Some(client) = self.clients.get_mut(&ext) {
                    client.initialized();
                }
                self.initialized_servers.insert(ext.clone(), true);
            }

            let events = if let Some(client) = self.clients.get(&ext) {
                client.poll_events()
            } else {
                continue;
            };

            for event in events {
                match event {
                    LspEvent::Diagnostics { file, diagnostics } => {
                        self.diagnostics.insert(file, diagnostics);
                    }
                    LspEvent::HoverResult { contents } => {
                        self.hover_info = Some(contents);
                    }
                    LspEvent::GotoDefinition { file, line, col } => {
                        self.pending_goto = Some((file, line, col));
                    }
                    LspEvent::Completion { items } => {
                        self.pending_completions = Some(items);
                    }
                    LspEvent::CodeActions { actions } => {
                        self.pending_code_actions = Some(actions);
                    }
                    LspEvent::References { locations } => {
                        self.pending_references = Some(locations);
                    }
                    LspEvent::FormatResult { edits } => {
                        self.pending_format = Some(edits);
                    }
                    LspEvent::Error(_) => {}
                }
            }
        }
    }

    pub fn get_diagnostics_for_line(&self, path: &Path, line: usize) -> Vec<&Diagnostic> {
        self.diagnostics
            .get(path)
            .map(|diags| diags.iter().filter(|d| d.line == line).collect())
            .unwrap_or_default()
    }

    pub fn get_diagnostics_for_file(&self, path: &Path) -> &[Diagnostic] {
        self.diagnostics
            .get(path)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn shutdown_all(&mut self) {
        for (_, client) in self.clients.iter_mut() {
            client.shutdown();
        }
    }

    pub fn has_server_for(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|ext| self.configs.contains_key(&ext.to_lowercase()))
            .unwrap_or(false)
    }
}

impl Drop for LspManager {
    fn drop(&mut self) {
        self.shutdown_all();
    }
}

// --- Helpers ---

fn parse_response(method: &str, result: &Value) -> LspEvent {
    match method {
        "textDocument/hover" => parse_hover_response(result),
        "textDocument/definition" => parse_definition_response(result),
        "textDocument/completion" => parse_completion_response(result),
        "textDocument/codeAction" => parse_code_actions_response(result),
        "textDocument/references" => parse_references_response(result),
        "textDocument/formatting" => parse_formatting_response(result),
        // For initialize and other responses, just ignore
        _ => LspEvent::Error(String::new()),
    }
}

fn parse_hover_response(result: &Value) -> LspEvent {
    if result.is_null() {
        return LspEvent::HoverResult {
            contents: String::new(),
        };
    }
    if let Some(contents) = result.get("contents") {
        let text = extract_hover_text(contents);
        return LspEvent::HoverResult { contents: text };
    }
    LspEvent::HoverResult {
        contents: String::new(),
    }
}

fn parse_definition_response(result: &Value) -> LspEvent {
    // Single location
    if let Some(uri) = result.get("uri") {
        if let (Some(uri_str), Some(range)) = (uri.as_str(), result.get("range")) {
            let file = uri_to_path(uri_str);
            let line = range
                .get("start")
                .and_then(|s| s.get("line"))
                .and_then(|l| l.as_u64())
                .unwrap_or(0) as usize;
            let col = range
                .get("start")
                .and_then(|s| s.get("character"))
                .and_then(|c| c.as_u64())
                .unwrap_or(0) as usize;
            return LspEvent::GotoDefinition { file, line, col };
        }
    }
    // Array of locations
    if let Some(arr) = result.as_array() {
        if let Some(first) = arr.first() {
            if let (Some(uri_str), Some(range)) = (
                first.get("uri").and_then(|u| u.as_str()),
                first.get("range"),
            ) {
                let file = uri_to_path(uri_str);
                let line = range
                    .get("start")
                    .and_then(|s| s.get("line"))
                    .and_then(|l| l.as_u64())
                    .unwrap_or(0) as usize;
                let col = range
                    .get("start")
                    .and_then(|s| s.get("character"))
                    .and_then(|c| c.as_u64())
                    .unwrap_or(0) as usize;
                return LspEvent::GotoDefinition { file, line, col };
            }
        }
    }
    LspEvent::Error("No definition found".to_string())
}

fn parse_completion_response(result: &Value) -> LspEvent {
    // CompletionList with "items" field
    if let Some(items) = result.get("items").and_then(|i| i.as_array()) {
        return LspEvent::Completion {
            items: parse_completion_items(items),
        };
    }
    // Direct array of completion items
    if let Some(arr) = result.as_array() {
        return LspEvent::Completion {
            items: parse_completion_items(arr),
        };
    }
    LspEvent::Completion { items: Vec::new() }
}

fn parse_code_actions_response(result: &Value) -> LspEvent {
    let actions = result
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    let title = item.get("title")?.as_str()?.to_string();
                    let kind = item.get("kind").and_then(|k| k.as_str()).map(|s| s.to_string());
                    let edit = item.get("edit").and_then(parse_workspace_edit);
                    Some(CodeAction { title, kind, edit })
                })
                .collect()
        })
        .unwrap_or_default();
    LspEvent::CodeActions { actions }
}

fn parse_workspace_edit(edit: &Value) -> Option<WorkspaceEdit> {
    let changes_obj = edit.get("changes")?.as_object()?;
    let mut changes = HashMap::new();
    for (uri, edits) in changes_obj {
        let path = uri_to_path(uri);
        let text_edits: Vec<TextEdit> = edits
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|e| {
                        let range: LspRange =
                            serde_json::from_value(e.get("range")?.clone()).ok()?;
                        let new_text = e.get("newText")?.as_str()?.to_string();
                        Some(TextEdit { range, new_text })
                    })
                    .collect()
            })
            .unwrap_or_default();
        changes.insert(path, text_edits);
    }
    Some(WorkspaceEdit { changes })
}

fn parse_references_response(result: &Value) -> LspEvent {
    let locations = result
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|loc| {
                    let uri = loc.get("uri")?.as_str()?;
                    let path = uri_to_path(uri);
                    let range = loc.get("range")?;
                    let line = range
                        .get("start")?
                        .get("line")?
                        .as_u64()? as usize;
                    let col = range
                        .get("start")?
                        .get("character")?
                        .as_u64()? as usize;
                    Some(ReferenceLocation { file: path, line, col })
                })
                .collect()
        })
        .unwrap_or_default();
    LspEvent::References { locations }
}

fn parse_formatting_response(result: &Value) -> LspEvent {
    let edits = result
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|e| {
                    let range: LspRange =
                        serde_json::from_value(e.get("range")?.clone()).ok()?;
                    let new_text = e.get("newText")?.as_str()?.to_string();
                    Some(TextEdit { range, new_text })
                })
                .collect()
        })
        .unwrap_or_default();
    LspEvent::FormatResult { edits }
}

fn parse_completion_items(items: &[Value]) -> Vec<CompletionItem> {
    items
        .iter()
        .filter_map(|item| {
            let label = item.get("label")?.as_str()?.to_string();
            let detail = item
                .get("detail")
                .and_then(|d| d.as_str())
                .map(|s| s.to_string());
            let insert_text = item
                .get("insertText")
                .and_then(|t| t.as_str())
                .map(|s| s.to_string());
            let kind_num = item.get("kind").and_then(|k| k.as_u64()).unwrap_or(1);
            let kind = match kind_num {
                2 => CompletionKind::Method,
                3 => CompletionKind::Function,
                4 => CompletionKind::Constructor,
                5 => CompletionKind::Field,
                6 => CompletionKind::Variable,
                7 => CompletionKind::Class,
                8 => CompletionKind::Interface,
                9 => CompletionKind::Module,
                10 => CompletionKind::Property,
                14 => CompletionKind::Keyword,
                15 => CompletionKind::Snippet,
                _ => CompletionKind::Other,
            };
            Some(CompletionItem {
                label,
                detail,
                insert_text,
                kind,
            })
        })
        .collect()
}

fn extract_hover_text(contents: &Value) -> String {
    // MarkupContent
    if let Some(value) = contents.get("value").and_then(|v| v.as_str()) {
        return value.to_string();
    }
    // Plain string
    if let Some(s) = contents.as_str() {
        return s.to_string();
    }
    // Array of MarkedString
    if let Some(arr) = contents.as_array() {
        return arr
            .iter()
            .filter_map(|item| {
                item.as_str().map(|s| s.to_string()).or_else(|| {
                    item.get("value")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                })
            })
            .collect::<Vec<_>>()
            .join("\n");
    }
    String::new()
}

fn path_to_uri(path: &Path) -> String {
    let abs = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    format!("file://{}", abs.display())
}

fn uri_to_path(uri: &str) -> PathBuf {
    if let Some(path_str) = uri.strip_prefix("file://") {
        PathBuf::from(path_str)
    } else {
        PathBuf::from(uri)
    }
}

fn extension_to_language_id(ext: &str) -> &str {
    match ext {
        "java" => "java",
        "kt" => "kotlin",
        "md" => "markdown",
        "proto" => "protobuf",
        "rs" => "rust",
        "py" => "python",
        "ts" | "tsx" => "typescript",
        "js" | "jsx" => "javascript",
        "go" => "go",
        "c" | "h" => "c",
        "cpp" | "hpp" | "cc" => "cpp",
        _ => ext,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_message() {
        let msg = r#"{"jsonrpc":"2.0"}"#;
        let encoded = encode_message(msg);
        let s = String::from_utf8(encoded).unwrap();
        assert!(s.starts_with("Content-Length: 17\r\n\r\n"));
        assert!(s.contains(r#"{"jsonrpc":"2.0"}"#));
    }

    #[test]
    fn test_decode_message() {
        let input = "Content-Length: 5\r\n\r\nhello";
        let mut reader = std::io::BufReader::new(input.as_bytes());
        let result = decode_message(&mut reader);
        assert_eq!(result, Some("hello".to_string()));
    }

    #[test]
    fn test_decode_message_empty() {
        let input = "";
        let mut reader = std::io::BufReader::new(input.as_bytes());
        let result = decode_message(&mut reader);
        assert!(result.is_none());
    }

    #[test]
    fn test_path_to_uri() {
        let path = Path::new("/tmp/test.rs");
        let uri = path_to_uri(path);
        assert!(uri.starts_with("file://"));
        assert!(uri.contains("test.rs"));
    }

    #[test]
    fn test_uri_to_path() {
        let path = uri_to_path("file:///tmp/test.rs");
        assert_eq!(path, PathBuf::from("/tmp/test.rs"));
    }

    #[test]
    fn test_extension_to_language_id() {
        assert_eq!(extension_to_language_id("rs"), "rust");
        assert_eq!(extension_to_language_id("py"), "python");
        assert_eq!(extension_to_language_id("java"), "java");
        assert_eq!(extension_to_language_id("ts"), "typescript");
        assert_eq!(extension_to_language_id("go"), "go");
        assert_eq!(extension_to_language_id("unknown"), "unknown");
    }

    #[test]
    fn test_parse_response_hover() {
        let result = json!({
            "contents": { "kind": "plaintext", "value": "fn main()" }
        });
        match parse_response("textDocument/hover", &result) {
            LspEvent::HoverResult { contents } => {
                assert_eq!(contents, "fn main()");
            }
            _ => panic!("Expected HoverResult"),
        }
    }

    #[test]
    fn test_parse_response_hover_string() {
        let result = json!({
            "contents": "simple hover"
        });
        match parse_response("textDocument/hover", &result) {
            LspEvent::HoverResult { contents } => {
                assert_eq!(contents, "simple hover");
            }
            _ => panic!("Expected HoverResult"),
        }
    }

    #[test]
    fn test_parse_response_goto_definition() {
        let result = json!({
            "uri": "file:///tmp/foo.rs",
            "range": {
                "start": { "line": 10, "character": 5 },
                "end": { "line": 10, "character": 15 }
            }
        });
        match parse_response("textDocument/definition", &result) {
            LspEvent::GotoDefinition { file, line, col } => {
                assert_eq!(file, PathBuf::from("/tmp/foo.rs"));
                assert_eq!(line, 10);
                assert_eq!(col, 5);
            }
            _ => panic!("Expected GotoDefinition"),
        }
    }

    #[test]
    fn test_parse_response_goto_definition_array() {
        let result = json!([{
            "uri": "file:///tmp/bar.rs",
            "range": {
                "start": { "line": 3, "character": 0 },
                "end": { "line": 3, "character": 10 }
            }
        }]);
        match parse_response("textDocument/definition", &result) {
            LspEvent::GotoDefinition { file, line, col } => {
                assert_eq!(file, PathBuf::from("/tmp/bar.rs"));
                assert_eq!(line, 3);
                assert_eq!(col, 0);
            }
            _ => panic!("Expected GotoDefinition"),
        }
    }

    #[test]
    fn test_parse_response_null() {
        let result = json!(null);
        match parse_response("textDocument/hover", &result) {
            LspEvent::HoverResult { contents } => {
                assert!(contents.is_empty());
            }
            _ => panic!("Expected empty HoverResult for null"),
        }
    }

    #[test]
    fn test_parse_code_actions() {
        let result = json!([{
            "title": "Import foo",
            "kind": "quickfix",
            "edit": {
                "changes": {
                    "file:///tmp/test.rs": [{
                        "range": {
                            "start": { "line": 0, "character": 0 },
                            "end": { "line": 0, "character": 0 }
                        },
                        "newText": "use foo;\n"
                    }]
                }
            }
        }]);
        match parse_response("textDocument/codeAction", &result) {
            LspEvent::CodeActions { actions } => {
                assert_eq!(actions.len(), 1);
                assert_eq!(actions[0].title, "Import foo");
                assert!(actions[0].edit.is_some());
            }
            _ => panic!("Expected CodeActions"),
        }
    }

    #[test]
    fn test_parse_references() {
        let result = json!([
            {
                "uri": "file:///tmp/a.rs",
                "range": { "start": { "line": 5, "character": 3 }, "end": { "line": 5, "character": 8 } }
            },
            {
                "uri": "file:///tmp/b.rs",
                "range": { "start": { "line": 10, "character": 0 }, "end": { "line": 10, "character": 5 } }
            }
        ]);
        match parse_response("textDocument/references", &result) {
            LspEvent::References { locations } => {
                assert_eq!(locations.len(), 2);
                assert_eq!(locations[0].file, PathBuf::from("/tmp/a.rs"));
                assert_eq!(locations[0].line, 5);
                assert_eq!(locations[1].file, PathBuf::from("/tmp/b.rs"));
            }
            _ => panic!("Expected References"),
        }
    }

    #[test]
    fn test_parse_formatting() {
        let result = json!([{
            "range": {
                "start": { "line": 0, "character": 0 },
                "end": { "line": 0, "character": 5 }
            },
            "newText": "hello"
        }]);
        match parse_response("textDocument/formatting", &result) {
            LspEvent::FormatResult { edits } => {
                assert_eq!(edits.len(), 1);
                assert_eq!(edits[0].new_text, "hello");
            }
            _ => panic!("Expected FormatResult"),
        }
    }

    #[test]
    fn test_extract_hover_text_markup_content() {
        let contents = json!({ "kind": "plaintext", "value": "hello" });
        assert_eq!(extract_hover_text(&contents), "hello");
    }

    #[test]
    fn test_extract_hover_text_string() {
        let contents = json!("plain string");
        assert_eq!(extract_hover_text(&contents), "plain string");
    }

    #[test]
    fn test_extract_hover_text_array() {
        let contents = json!(["first", { "value": "second" }]);
        assert_eq!(extract_hover_text(&contents), "first\nsecond");
    }

    #[test]
    fn test_lsp_manager_no_config() {
        let manager = LspManager::new(HashMap::new(), Path::new("/tmp"));
        assert!(!manager.has_server_for(Path::new("test.rs")));
        assert!(manager
            .get_diagnostics_for_file(Path::new("test.rs"))
            .is_empty());
    }

    #[test]
    fn test_lsp_manager_has_server() {
        let mut configs = HashMap::new();
        configs.insert(
            "rs".to_string(),
            LspServerConfig {
                command: "rust-analyzer".to_string(),
                args: vec![],
            },
        );
        let manager = LspManager::new(configs, Path::new("/tmp"));
        assert!(manager.has_server_for(Path::new("test.rs")));
        assert!(!manager.has_server_for(Path::new("test.py")));
    }

    #[test]
    fn test_diagnostic_severity() {
        assert_ne!(DiagnosticSeverity::Error, DiagnosticSeverity::Warning);
        assert_eq!(DiagnosticSeverity::Error, DiagnosticSeverity::Error);
    }
}
