use crate::command::{Command, FocusPane, SimpleCommand};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub modifiers: Modifiers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    Char(char),
    Enter,
    Backspace,
    Delete,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Tab,
    Esc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Modifiers {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub super_key: bool,
}

impl Modifiers {
    pub fn none() -> Self {
        Self {
            ctrl: false,
            shift: false,
            alt: false,
            super_key: false,
        }
    }

    pub fn ctrl() -> Self {
        Self {
            ctrl: true,
            shift: false,
            alt: false,
            super_key: false,
        }
    }

    pub fn ctrl_shift() -> Self {
        Self {
            ctrl: true,
            shift: true,
            alt: false,
            super_key: false,
        }
    }

    pub fn alt() -> Self {
        Self {
            ctrl: false,
            shift: false,
            alt: true,
            super_key: false,
        }
    }

    pub fn super_key() -> Self {
        Self {
            ctrl: false,
            shift: false,
            alt: false,
            super_key: true,
        }
    }

    pub fn super_shift() -> Self {
        Self {
            ctrl: false,
            shift: true,
            alt: false,
            super_key: true,
        }
    }

    /// Return the equivalent with super_key instead of ctrl.
    pub fn as_super(&self) -> Self {
        Self {
            ctrl: false,
            shift: self.shift,
            alt: self.alt,
            super_key: true,
        }
    }
}

// --- Keymap preset ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum KeymapPreset {
    #[default]
    Default,
    Mac,
}

// --- JSON schema types ---

#[derive(Deserialize)]
struct KeybindingsFile {
    #[serde(default)]
    editor: Vec<BindingEntry>,
    #[serde(default)]
    explorer: Vec<BindingEntry>,
    #[serde(default)]
    search: Vec<BindingEntry>,
    #[serde(default)]
    fuzzy: Vec<BindingEntry>,
    #[serde(default)]
    goto_line: Vec<BindingEntry>,
    #[serde(default)]
    completion: Vec<BindingEntry>,
    #[serde(default)]
    code_action: Vec<BindingEntry>,
    #[serde(default)]
    references: Vec<BindingEntry>,
    #[serde(default)]
    explorer_input: Vec<BindingEntry>,
}

#[derive(Deserialize)]
struct BindingEntry {
    key: String,
    command: SimpleCommand,
}

// --- KeymapConfig ---

pub struct KeymapConfig {
    editor: HashMap<KeyEvent, Command>,
    explorer: HashMap<KeyEvent, Command>,
    search: HashMap<KeyEvent, Command>,
    fuzzy: HashMap<KeyEvent, Command>,
    goto_line: HashMap<KeyEvent, Command>,
    completion: HashMap<KeyEvent, Command>,
    code_action: HashMap<KeyEvent, Command>,
    references: HashMap<KeyEvent, Command>,
    explorer_input: HashMap<KeyEvent, Command>,
}

impl KeymapConfig {
    /// Load keybindings from a JSON file, falling back to defaults if the file
    /// is missing or malformed.
    pub fn load(path: &Path, preset: KeymapPreset) -> Self {
        let file_path = path.join("keybindings.json");
        match std::fs::read_to_string(&file_path) {
            Ok(contents) => match serde_json::from_str::<KeybindingsFile>(&contents) {
                Ok(file) => Self::from_file(&file, preset),
                Err(_) => Self::defaults(preset),
            },
            Err(_) => Self::defaults(preset),
        }
    }

    /// Build from the JSON file, using defaults as the base and overriding
    /// with any bindings defined in the file.
    fn from_file(file: &KeybindingsFile, preset: KeymapPreset) -> Self {
        let mut config = Self::defaults(preset);

        for entry in &file.editor {
            if let Some(key) = parse_key_string(&entry.key) {
                config.editor.insert(key, entry.command.into_command());
            }
        }
        for entry in &file.explorer {
            if let Some(key) = parse_key_string(&entry.key) {
                config.explorer.insert(key, entry.command.into_command());
            }
        }
        for entry in &file.search {
            if let Some(key) = parse_key_string(&entry.key) {
                config.search.insert(key, entry.command.into_command());
            }
        }
        for entry in &file.fuzzy {
            if let Some(key) = parse_key_string(&entry.key) {
                config.fuzzy.insert(key, entry.command.into_command());
            }
        }
        for entry in &file.goto_line {
            if let Some(key) = parse_key_string(&entry.key) {
                config.goto_line.insert(key, entry.command.into_command());
            }
        }
        for entry in &file.completion {
            if let Some(key) = parse_key_string(&entry.key) {
                config.completion.insert(key, entry.command.into_command());
            }
        }
        for entry in &file.code_action {
            if let Some(key) = parse_key_string(&entry.key) {
                config.code_action.insert(key, entry.command.into_command());
            }
        }
        for entry in &file.references {
            if let Some(key) = parse_key_string(&entry.key) {
                config.references.insert(key, entry.command.into_command());
            }
        }
        for entry in &file.explorer_input {
            if let Some(key) = parse_key_string(&entry.key) {
                config
                    .explorer_input
                    .insert(key, entry.command.into_command());
            }
        }

        config
    }

    /// Hardcoded defaults matching the original keybindings.
    /// When `preset` is `Mac`, every Ctrl+X binding is also registered as Cmd+X.
    pub fn defaults(preset: KeymapPreset) -> Self {
        let mut editor = HashMap::new();
        let mut explorer = HashMap::new();
        let mut search = HashMap::new();

        // Editor bindings
        editor.insert(key('p', Modifiers::ctrl()), Command::FuzzyOpen);
        editor.insert(key('s', Modifiers::ctrl()), Command::Save);
        editor.insert(key('q', Modifiers::ctrl()), Command::Quit);
        editor.insert(key('w', Modifiers::ctrl()), Command::CloseTab);
        editor.insert(key('b', Modifiers::ctrl()), Command::ToggleExplorer);
        editor.insert(key('f', Modifiers::ctrl()), Command::SearchInFile);
        editor.insert(key('z', Modifiers::ctrl()), Command::Undo);
        editor.insert(
            key('f', Modifiers::ctrl_shift()),
            Command::SearchAcrossFiles,
        );
        editor.insert(
            KeyEvent {
                code: KeyCode::PageDown,
                modifiers: Modifiers::ctrl(),
            },
            Command::NextTab,
        );
        editor.insert(
            KeyEvent {
                code: KeyCode::PageUp,
                modifiers: Modifiers::ctrl(),
            },
            Command::PrevTab,
        );
        editor.insert(
            KeyEvent {
                code: KeyCode::Left,
                modifiers: Modifiers::alt(),
            },
            Command::PrevTab,
        );
        editor.insert(
            KeyEvent {
                code: KeyCode::Right,
                modifiers: Modifiers::alt(),
            },
            Command::NextTab,
        );
        editor.insert(
            KeyEvent {
                code: KeyCode::Home,
                modifiers: Modifiers::ctrl(),
            },
            Command::MoveToFileStart,
        );
        editor.insert(
            KeyEvent {
                code: KeyCode::End,
                modifiers: Modifiers::ctrl(),
            },
            Command::MoveToFileEnd,
        );
        editor.insert(special(KeyCode::Up), Command::MoveUp);
        editor.insert(special(KeyCode::Down), Command::MoveDown);
        editor.insert(special(KeyCode::Left), Command::MoveLeft);
        editor.insert(special(KeyCode::Right), Command::MoveRight);
        editor.insert(
            KeyEvent {
                code: KeyCode::Left,
                modifiers: Modifiers::ctrl(),
            },
            Command::MoveWordLeft,
        );
        editor.insert(
            KeyEvent {
                code: KeyCode::Right,
                modifiers: Modifiers::ctrl(),
            },
            Command::MoveWordRight,
        );
        editor.insert(key('g', Modifiers::ctrl()), Command::GoToLineOpen);
        editor.insert(key('h', Modifiers::ctrl()), Command::LspHover);
        editor.insert(key('d', Modifiers::ctrl()), Command::LspGotoDefinition);
        editor.insert(key(' ', Modifiers::ctrl()), Command::LspComplete);
        editor.insert(key('.', Modifiers::ctrl()), Command::LspCodeAction);
        editor.insert(
            key('r', Modifiers::ctrl_shift()),
            Command::LspFindReferences,
        );
        editor.insert(
            key('i', Modifiers::ctrl_shift()),
            Command::LspFormat,
        );
        // Folding: Ctrl+[ to toggle, Ctrl+] to unfold all
        editor.insert(key('[', Modifiers::ctrl()), Command::ToggleFold);
        editor.insert(key(']', Modifiers::ctrl()), Command::UnfoldAll);
        editor.insert(special(KeyCode::Home), Command::MoveToLineStart);
        editor.insert(special(KeyCode::End), Command::MoveToLineEnd);
        editor.insert(special(KeyCode::PageUp), Command::PageUp);
        editor.insert(special(KeyCode::PageDown), Command::PageDown);
        editor.insert(special(KeyCode::Enter), Command::InsertNewline);
        editor.insert(special(KeyCode::Backspace), Command::DeleteBack);
        editor.insert(special(KeyCode::Delete), Command::DeleteForward);
        editor.insert(special(KeyCode::Tab), Command::InsertChar('\t'));

        // Explorer bindings
        explorer.insert(key('q', Modifiers::ctrl()), Command::Quit);
        explorer.insert(key('b', Modifiers::ctrl()), Command::ToggleExplorer);
        explorer.insert(key('f', Modifiers::ctrl()), Command::SearchInFile);
        explorer.insert(
            key('f', Modifiers::ctrl_shift()),
            Command::SearchAcrossFiles,
        );
        explorer.insert(special(KeyCode::Up), Command::ExplorerUp);
        explorer.insert(special(KeyCode::Down), Command::ExplorerDown);
        explorer.insert(special(KeyCode::Enter), Command::ExplorerEnter);
        explorer.insert(special(KeyCode::Tab), Command::FocusEditor);
        explorer.insert(special(KeyCode::Esc), Command::FocusEditor);
        explorer.insert(key('n', Modifiers::none()), Command::ExplorerNewFile);
        explorer.insert(key('N', Modifiers::none()), Command::ExplorerNewFolder);
        explorer.insert(key('r', Modifiers::none()), Command::ExplorerRename);
        explorer.insert(key('d', Modifiers::none()), Command::ExplorerDelete);

        // Search bindings
        search.insert(special(KeyCode::Esc), Command::SearchClose);
        search.insert(special(KeyCode::Enter), Command::SearchNext);
        search.insert(key('n', Modifiers::ctrl()), Command::SearchNext);
        search.insert(key('p', Modifiers::ctrl()), Command::SearchPrev);
        search.insert(special(KeyCode::Backspace), Command::SearchBackspace);

        // Fuzzy finder bindings
        let mut fuzzy = HashMap::new();
        fuzzy.insert(special(KeyCode::Esc), Command::FuzzyClose);
        fuzzy.insert(special(KeyCode::Enter), Command::FuzzyConfirm);
        fuzzy.insert(special(KeyCode::Up), Command::FuzzyUp);
        fuzzy.insert(special(KeyCode::Down), Command::FuzzyDown);
        fuzzy.insert(key('p', Modifiers::ctrl()), Command::FuzzyClose);
        fuzzy.insert(special(KeyCode::Backspace), Command::FuzzyBackspace);

        // Go-to-line bindings
        let mut goto_line = HashMap::new();
        goto_line.insert(special(KeyCode::Esc), Command::GoToLineClose);
        goto_line.insert(special(KeyCode::Enter), Command::GoToLineConfirm);
        goto_line.insert(special(KeyCode::Backspace), Command::GoToLineBackspace);

        // Completion bindings
        let mut completion = HashMap::new();
        completion.insert(special(KeyCode::Esc), Command::CompletionClose);
        completion.insert(special(KeyCode::Enter), Command::CompletionConfirm);
        completion.insert(special(KeyCode::Tab), Command::CompletionConfirm);
        completion.insert(special(KeyCode::Up), Command::CompletionUp);
        completion.insert(special(KeyCode::Down), Command::CompletionDown);

        // Code action bindings
        let mut code_action = HashMap::new();
        code_action.insert(special(KeyCode::Esc), Command::CodeActionClose);
        code_action.insert(special(KeyCode::Enter), Command::CodeActionConfirm);
        code_action.insert(special(KeyCode::Up), Command::CodeActionUp);
        code_action.insert(special(KeyCode::Down), Command::CodeActionDown);

        // References bindings
        let mut references = HashMap::new();
        references.insert(special(KeyCode::Esc), Command::ReferencesClose);
        references.insert(special(KeyCode::Enter), Command::ReferencesConfirm);
        references.insert(special(KeyCode::Up), Command::ReferencesUp);
        references.insert(special(KeyCode::Down), Command::ReferencesDown);

        // Explorer input bindings
        let mut explorer_input = HashMap::new();
        explorer_input.insert(special(KeyCode::Esc), Command::ExplorerCancelInput);
        explorer_input.insert(special(KeyCode::Enter), Command::ExplorerConfirmInput);
        explorer_input.insert(special(KeyCode::Backspace), Command::ExplorerInputBackspace);

        let mut config = Self {
            editor,
            explorer,
            search,
            fuzzy,
            goto_line,
            completion,
            code_action,
            references,
            explorer_input,
        };

        if preset == KeymapPreset::Mac {
            config.add_cmd_aliases();
        }

        config
    }

    /// For every Ctrl+X binding in every pane, add a corresponding Cmd+X binding
    /// (Super key). This lets Mac users use Cmd as the primary modifier.
    fn add_cmd_aliases(&mut self) {
        fn add_super_aliases(map: &mut HashMap<KeyEvent, Command>) {
            let aliases: Vec<(KeyEvent, Command)> = map
                .iter()
                .filter(|(k, _)| k.modifiers.ctrl && !k.modifiers.super_key)
                .map(|(k, v)| {
                    let super_event = KeyEvent {
                        code: k.code,
                        modifiers: k.modifiers.as_super(),
                    };
                    (super_event, v.clone())
                })
                .collect();
            for (k, v) in aliases {
                map.entry(k).or_insert(v);
            }
        }

        add_super_aliases(&mut self.editor);
        add_super_aliases(&mut self.explorer);
        add_super_aliases(&mut self.search);
        add_super_aliases(&mut self.fuzzy);
        add_super_aliases(&mut self.goto_line);
        add_super_aliases(&mut self.completion);
        add_super_aliases(&mut self.code_action);
        add_super_aliases(&mut self.references);
        add_super_aliases(&mut self.explorer_input);
    }

    /// Map a key event to a command, using the bindings for the given focus pane.
    /// Character input (InsertChar / SearchInput) falls back to hardcoded logic
    /// since those can't be represented in JSON.
    pub fn map_key(&self, event: KeyEvent, focus: FocusPane) -> Command {
        let table = match focus {
            FocusPane::Editor => &self.editor,
            FocusPane::Explorer => &self.explorer,
            FocusPane::SearchBar => &self.search,
            FocusPane::FuzzyFinder => &self.fuzzy,
            FocusPane::GoToLine => &self.goto_line,
            FocusPane::Completion => &self.completion,
            FocusPane::CodeAction => &self.code_action,
            FocusPane::References => &self.references,
            FocusPane::ExplorerInput => &self.explorer_input,
        };

        if let Some(cmd) = table.get(&event) {
            return cmd.clone();
        }

        // Hardcoded character fallbacks — only plain keypresses (no modifier keys)
        let has_modifier = event.modifiers.ctrl || event.modifiers.alt || event.modifiers.super_key;
        match focus {
            FocusPane::Editor => {
                if let KeyCode::Char(c) = event.code {
                    if !has_modifier {
                        return Command::InsertChar(c);
                    }
                }
            }
            FocusPane::SearchBar => {
                if let KeyCode::Char(c) = event.code {
                    if !has_modifier {
                        return Command::SearchInput(c);
                    }
                }
            }
            FocusPane::FuzzyFinder => {
                if let KeyCode::Char(c) = event.code {
                    if !has_modifier {
                        return Command::FuzzyInput(c);
                    }
                }
            }
            FocusPane::GoToLine => {
                if let KeyCode::Char(c) = event.code {
                    if !has_modifier && c.is_ascii_digit() {
                        return Command::GoToLineInput(c);
                    }
                }
            }
            FocusPane::ExplorerInput => {
                if let KeyCode::Char(c) = event.code {
                    if !has_modifier {
                        return Command::ExplorerInputChar(c);
                    }
                }
            }
            _ => {}
        }

        Command::None
    }
}

// --- Helpers ---

fn key(ch: char, modifiers: Modifiers) -> KeyEvent {
    KeyEvent {
        code: KeyCode::Char(ch),
        modifiers,
    }
}

fn special(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: Modifiers::none(),
    }
}

/// Parse a key string like "ctrl+shift+f" or "pagedown" into a KeyEvent.
fn parse_key_string(s: &str) -> Option<KeyEvent> {
    let parts: Vec<&str> = s.split('+').collect();
    let mut ctrl = false;
    let mut shift = false;
    let mut alt = false;

    let mut super_key = false;

    // All parts except the last are modifiers
    for &part in &parts[..parts.len() - 1] {
        match part.to_lowercase().as_str() {
            "ctrl" => ctrl = true,
            "shift" => shift = true,
            "alt" => alt = true,
            "cmd" | "super" | "command" => super_key = true,
            _ => return None,
        }
    }

    let key_name = parts.last()?.to_lowercase();
    let code = match key_name.as_str() {
        "enter" => KeyCode::Enter,
        "backspace" => KeyCode::Backspace,
        "delete" => KeyCode::Delete,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "pageup" => KeyCode::PageUp,
        "pagedown" => KeyCode::PageDown,
        "tab" => KeyCode::Tab,
        "esc" | "escape" => KeyCode::Esc,
        s if s.len() == 1 => KeyCode::Char(s.chars().next()?),
        _ => return None,
    };

    Some(KeyEvent {
        code,
        modifiers: Modifiers {
            ctrl,
            shift,
            alt,
            super_key,
        },
    })
}

// Keep the free function for backward compatibility during transition
pub fn map_key(event: KeyEvent, focus: FocusPane) -> Command {
    KeymapConfig::defaults(KeymapPreset::Default).map_key(event, focus)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_key_simple_char() {
        let ke = parse_key_string("s").unwrap();
        assert_eq!(ke.code, KeyCode::Char('s'));
        assert!(!ke.modifiers.ctrl);
        assert!(!ke.modifiers.shift);
        assert!(!ke.modifiers.alt);
    }

    #[test]
    fn test_parse_key_ctrl() {
        let ke = parse_key_string("ctrl+s").unwrap();
        assert_eq!(ke.code, KeyCode::Char('s'));
        assert!(ke.modifiers.ctrl);
    }

    #[test]
    fn test_parse_key_ctrl_shift() {
        let ke = parse_key_string("ctrl+shift+f").unwrap();
        assert_eq!(ke.code, KeyCode::Char('f'));
        assert!(ke.modifiers.ctrl);
        assert!(ke.modifiers.shift);
    }

    #[test]
    fn test_parse_key_special() {
        assert_eq!(parse_key_string("enter").unwrap().code, KeyCode::Enter);
        assert_eq!(parse_key_string("esc").unwrap().code, KeyCode::Esc);
        assert_eq!(parse_key_string("escape").unwrap().code, KeyCode::Esc);
        assert_eq!(parse_key_string("pageup").unwrap().code, KeyCode::PageUp);
        assert_eq!(parse_key_string("tab").unwrap().code, KeyCode::Tab);
    }

    #[test]
    fn test_parse_key_invalid() {
        assert!(parse_key_string("invalidkey").is_none());
        assert!(parse_key_string("badmod+s").is_none());
    }

    #[test]
    fn test_parse_key_alt() {
        let ke = parse_key_string("alt+left").unwrap();
        assert_eq!(ke.code, KeyCode::Left);
        assert!(ke.modifiers.alt);
    }

    #[test]
    fn test_defaults_editor_ctrl_s_is_save() {
        let config = KeymapConfig::defaults(KeymapPreset::Default);
        let event = KeyEvent {
            code: KeyCode::Char('s'),
            modifiers: Modifiers::ctrl(),
        };
        assert_eq!(config.map_key(event, FocusPane::Editor), Command::Save);
    }

    #[test]
    fn test_defaults_editor_arrow_keys() {
        let config = KeymapConfig::defaults(KeymapPreset::Default);
        let up = KeyEvent {
            code: KeyCode::Up,
            modifiers: Modifiers::none(),
        };
        assert_eq!(config.map_key(up, FocusPane::Editor), Command::MoveUp);
    }

    #[test]
    fn test_editor_char_fallback() {
        let config = KeymapConfig::defaults(KeymapPreset::Default);
        let event = KeyEvent {
            code: KeyCode::Char('x'),
            modifiers: Modifiers::none(),
        };
        assert_eq!(
            config.map_key(event, FocusPane::Editor),
            Command::InsertChar('x')
        );
    }

    #[test]
    fn test_search_char_fallback() {
        let config = KeymapConfig::defaults(KeymapPreset::Default);
        let event = KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: Modifiers::none(),
        };
        assert_eq!(
            config.map_key(event, FocusPane::SearchBar),
            Command::SearchInput('a')
        );
    }

    #[test]
    fn test_fuzzy_char_fallback() {
        let config = KeymapConfig::defaults(KeymapPreset::Default);
        let event = KeyEvent {
            code: KeyCode::Char('m'),
            modifiers: Modifiers::none(),
        };
        assert_eq!(
            config.map_key(event, FocusPane::FuzzyFinder),
            Command::FuzzyInput('m')
        );
    }

    #[test]
    fn test_explorer_enter() {
        let config = KeymapConfig::defaults(KeymapPreset::Default);
        let event = KeyEvent {
            code: KeyCode::Enter,
            modifiers: Modifiers::none(),
        };
        assert_eq!(
            config.map_key(event, FocusPane::Explorer),
            Command::ExplorerEnter
        );
    }

    #[test]
    fn test_unbound_key_returns_none() {
        let config = KeymapConfig::defaults(KeymapPreset::Default);
        let event = KeyEvent {
            code: KeyCode::Char('x'),
            modifiers: Modifiers {
                ctrl: true,
                shift: true,
                alt: true,
                super_key: false,
            },
        };
        assert_eq!(config.map_key(event, FocusPane::Editor), Command::None);
    }

    #[test]
    fn test_load_from_json() {
        let dir = tempfile::tempdir().unwrap();
        let json = r#"{
            "editor": [
                { "key": "ctrl+k", "command": "Save" }
            ],
            "explorer": [],
            "search": []
        }"#;
        std::fs::write(dir.path().join("keybindings.json"), json).unwrap();
        let config = KeymapConfig::load(dir.path(), KeymapPreset::Default);

        // Custom binding works
        let event = KeyEvent {
            code: KeyCode::Char('k'),
            modifiers: Modifiers::ctrl(),
        };
        assert_eq!(config.map_key(event, FocusPane::Editor), Command::Save);

        // Default binding still works
        let event = KeyEvent {
            code: KeyCode::Up,
            modifiers: Modifiers::none(),
        };
        assert_eq!(config.map_key(event, FocusPane::Editor), Command::MoveUp);
    }

    #[test]
    fn test_load_missing_file_uses_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let config = KeymapConfig::load(dir.path(), KeymapPreset::Default);
        let event = KeyEvent {
            code: KeyCode::Char('s'),
            modifiers: Modifiers::ctrl(),
        };
        assert_eq!(config.map_key(event, FocusPane::Editor), Command::Save);
    }

    #[test]
    fn test_load_malformed_json_uses_defaults() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("keybindings.json"), "not json!").unwrap();
        let config = KeymapConfig::load(dir.path(), KeymapPreset::Default);
        let event = KeyEvent {
            code: KeyCode::Char('s'),
            modifiers: Modifiers::ctrl(),
        };
        assert_eq!(config.map_key(event, FocusPane::Editor), Command::Save);
    }

    #[test]
    fn test_mac_preset_cmd_s_saves() {
        let config = KeymapConfig::defaults(KeymapPreset::Mac);
        // Cmd+S should map to Save
        let event = KeyEvent {
            code: KeyCode::Char('s'),
            modifiers: Modifiers::super_key(),
        };
        assert_eq!(config.map_key(event, FocusPane::Editor), Command::Save);
        // Ctrl+S should still work too
        let event = KeyEvent {
            code: KeyCode::Char('s'),
            modifiers: Modifiers::ctrl(),
        };
        assert_eq!(config.map_key(event, FocusPane::Editor), Command::Save);
    }

    #[test]
    fn test_mac_preset_cmd_bindings() {
        let config = KeymapConfig::defaults(KeymapPreset::Mac);
        // Cmd+P -> FuzzyOpen
        let event = KeyEvent {
            code: KeyCode::Char('p'),
            modifiers: Modifiers::super_key(),
        };
        assert_eq!(config.map_key(event, FocusPane::Editor), Command::FuzzyOpen);
        // Cmd+Z -> Undo
        let event = KeyEvent {
            code: KeyCode::Char('z'),
            modifiers: Modifiers::super_key(),
        };
        assert_eq!(config.map_key(event, FocusPane::Editor), Command::Undo);
        // Cmd+F -> SearchInFile
        let event = KeyEvent {
            code: KeyCode::Char('f'),
            modifiers: Modifiers::super_key(),
        };
        assert_eq!(
            config.map_key(event, FocusPane::Editor),
            Command::SearchInFile
        );
        // Cmd+Shift+F -> SearchAcrossFiles
        let event = KeyEvent {
            code: KeyCode::Char('f'),
            modifiers: Modifiers::super_shift(),
        };
        assert_eq!(
            config.map_key(event, FocusPane::Editor),
            Command::SearchAcrossFiles
        );
    }

    #[test]
    fn test_default_preset_no_cmd_bindings() {
        let config = KeymapConfig::defaults(KeymapPreset::Default);
        let event = KeyEvent {
            code: KeyCode::Char('s'),
            modifiers: Modifiers::super_key(),
        };
        assert_eq!(config.map_key(event, FocusPane::Editor), Command::None);
    }

    #[test]
    fn test_parse_key_cmd() {
        let ke = parse_key_string("cmd+s").unwrap();
        assert_eq!(ke.code, KeyCode::Char('s'));
        assert!(ke.modifiers.super_key);
        assert!(!ke.modifiers.ctrl);

        let ke = parse_key_string("super+f").unwrap();
        assert!(ke.modifiers.super_key);
    }
}
