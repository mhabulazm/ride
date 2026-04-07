use ride_core::command::{Command, FocusPane};
use ride_core::explorer::Explorer;
use ride_core::folding::FoldState;
use ride_core::fuzzy::FuzzyFinder;
use ride_core::highlight::{self, HighlighterType};
use ride_core::highlight::treesitter_hl::TreeSitterHighlighter;
use ride_core::keymap::{self, KeymapConfig};
use ride_core::lsp::{CompletionItem, LspManager};
use ride_core::search::SearchState;
use ride_core::settings::Settings;
use ride_core::tab::TabManager;
use ride_core::theme::Theme;
use std::path::{Path, PathBuf};
use std::time::Instant;

pub struct App {
    pub tabs: TabManager,
    pub explorer: Explorer,
    pub search: SearchState,
    pub focus: FocusPane,
    pub running: bool,
    pub status_message: String,
    pub viewport_height: usize,
    pub working_dir: PathBuf,
    pub highlighter_types: Vec<HighlighterType>,
    pub ts_highlighters: Vec<Option<TreeSitterHighlighter>>,
    pub fold_states: Vec<FoldState>,
    pub keymap: KeymapConfig,
    pub fuzzy: FuzzyFinder,
    pub goto_line_input: String,
    pub settings: Settings,
    pub last_autosave: Instant,
    pub lsp: LspManager,
    pub hover_display: Option<String>,
    pub completion_items: Vec<CompletionItem>,
    pub completion_index: usize,
    pub completion_active: bool,
    pub theme: Theme,
    doc_versions: std::collections::HashMap<PathBuf, i32>,
}

impl App {
    pub fn new(path: &Path) -> Self {
        let (working_dir, initial_file) = if path.is_dir() {
            (path.to_path_buf(), None)
        } else {
            let dir = path.parent().unwrap_or(Path::new(".")).to_path_buf();
            (dir, Some(path.to_path_buf()))
        };

        let explorer = Explorer::new(&working_dir);
        let settings = Settings::load(&working_dir);
        let keymap = KeymapConfig::load(&working_dir, settings.keymap_preset);
        let fuzzy = FuzzyFinder::new(&working_dir);
        let theme = settings.resolve_theme();
        let lsp = LspManager::new(settings.lsp.clone(), &working_dir);
        let mut app = Self {
            tabs: TabManager::new(),
            explorer,
            search: SearchState::new(),
            focus: FocusPane::Editor,
            running: true,
            status_message: String::new(),
            viewport_height: 24,
            working_dir,
            highlighter_types: Vec::new(),
            ts_highlighters: Vec::new(),
            fold_states: Vec::new(),
            keymap,
            fuzzy,
            goto_line_input: String::new(),
            settings,
            last_autosave: Instant::now(),
            lsp,
            hover_display: None,
            completion_items: Vec::new(),
            completion_index: 0,
            completion_active: false,
            theme,
            doc_versions: std::collections::HashMap::new(),
        };

        if let Some(file) = initial_file {
            app.open_file(&file);
        }

        app
    }

    pub fn open_file(&mut self, path: &Path) {
        match self.tabs.open_file(path) {
            Ok(()) => {
                let hl_type = highlight::detect_highlighter(path);
                // Ensure highlighter_types vec matches tabs
                while self.highlighter_types.len() < self.tabs.tabs.len() {
                    self.highlighter_types.push(HighlighterType::Plain);
                }
                while self.ts_highlighters.len() < self.tabs.tabs.len() {
                    self.ts_highlighters.push(None);
                }
                while self.fold_states.len() < self.tabs.tabs.len() {
                    self.fold_states.push(FoldState::new());
                }
                self.highlighter_types[self.tabs.active] = hl_type;
                // Initialize tree-sitter highlighter if applicable
                if let HighlighterType::TreeSitter(lang) = hl_type {
                    if let Some(mut hl) = TreeSitterHighlighter::new(lang) {
                        if let Some(buf) = self.tabs.active_buffer() {
                            let source = buf.rope.to_string();
                            hl.parse(&source);
                            // Initialize fold regions
                            if let Some(tree) = hl.tree() {
                                let lang_name = hl.lang_name().to_string();
                                if let Some(fold_state) = self.fold_states.get_mut(self.tabs.active) {
                                    fold_state.update_regions_from_tree(tree, &source, &lang_name);
                                }
                            }
                        }
                        self.ts_highlighters[self.tabs.active] = Some(hl);
                    }
                } else {
                    self.ts_highlighters[self.tabs.active] = None;
                }
                self.focus = FocusPane::Editor;
                self.status_message = format!("Opened {}", path.display());
                // Notify LSP
                if self.lsp.has_server_for(path) {
                    if let Some(buf) = self.tabs.active_buffer() {
                        let text = buf.rope.to_string();
                        self.lsp.did_open(path, &text);
                        self.doc_versions.insert(path.to_path_buf(), 1);
                    }
                }
            }
            Err(e) => {
                self.status_message = format!("Error: {}", e);
            }
        }
    }

    pub fn active_highlighter(&self) -> HighlighterType {
        self.highlighter_types
            .get(self.tabs.active)
            .copied()
            .unwrap_or(HighlighterType::Plain)
    }

    /// Reparse tree-sitter for the active tab if it has one.
    /// Also updates fold regions from the parse tree.
    pub fn reparse_tree_sitter(&mut self) {
        if let Some(Some(ref mut hl)) = self.ts_highlighters.get_mut(self.tabs.active) {
            if let Some(buf) = self.tabs.tabs.get(self.tabs.active) {
                if buf.dirty {
                    let source = buf.rope.to_string();
                    hl.parse(&source);
                    // Update fold regions
                    if let Some(tree) = hl.tree() {
                        let lang_name = hl.lang_name().to_string();
                        if let Some(fold_state) = self.fold_states.get_mut(self.tabs.active) {
                            fold_state.update_regions_from_tree(tree, &source, &lang_name);
                        }
                    }
                }
            }
        }
    }

    /// Get tree-sitter highlight spans for a line, if available.
    pub fn ts_highlight_line(&self, line_idx: usize) -> Option<Vec<highlight::HighlightSpan>> {
        let ts_hl = self.ts_highlighters.get(self.tabs.active)?.as_ref()?;
        let buf = self.tabs.tabs.get(self.tabs.active)?;
        let source = buf.rope.to_string();
        Some(ts_hl.highlight_line(&source, line_idx))
    }

    pub fn handle_command(&mut self, cmd: Command) {
        match cmd {
            Command::None => {}

            // Cursor movement
            Command::MoveUp => {
                if let Some(buf) = self.tabs.active_buffer_mut() {
                    buf.move_up();
                }
            }
            Command::MoveDown => {
                if let Some(buf) = self.tabs.active_buffer_mut() {
                    buf.move_down();
                }
            }
            Command::MoveLeft => {
                if let Some(buf) = self.tabs.active_buffer_mut() {
                    buf.move_left();
                }
            }
            Command::MoveRight => {
                if let Some(buf) = self.tabs.active_buffer_mut() {
                    buf.move_right();
                }
            }
            Command::MoveWordLeft => {
                if let Some(buf) = self.tabs.active_buffer_mut() {
                    buf.move_word_left();
                }
            }
            Command::MoveWordRight => {
                if let Some(buf) = self.tabs.active_buffer_mut() {
                    buf.move_word_right();
                }
            }
            Command::MoveToLineStart => {
                if let Some(buf) = self.tabs.active_buffer_mut() {
                    buf.move_to_line_start();
                }
            }
            Command::MoveToLineEnd => {
                if let Some(buf) = self.tabs.active_buffer_mut() {
                    buf.move_to_line_end();
                }
            }
            Command::MoveToFileStart => {
                if let Some(buf) = self.tabs.active_buffer_mut() {
                    buf.move_to_file_start();
                }
            }
            Command::MoveToFileEnd => {
                if let Some(buf) = self.tabs.active_buffer_mut() {
                    buf.move_to_file_end();
                }
            }
            Command::PageUp => {
                let h = self.viewport_height;
                if let Some(buf) = self.tabs.active_buffer_mut() {
                    buf.page_up(h);
                }
            }
            Command::PageDown => {
                let h = self.viewport_height;
                if let Some(buf) = self.tabs.active_buffer_mut() {
                    buf.page_down(h);
                }
            }

            // Editing
            Command::InsertChar(c) => {
                if let Some(buf) = self.tabs.active_buffer_mut() {
                    buf.insert_char(c);
                }
                // Auto-trigger completion on '.' or ':' (for '::')
                if c == '.' || c == ':' {
                    if let Some(buf) = self.tabs.active_buffer() {
                        if let Some(ref path) = buf.file_path.clone() {
                            if self.lsp.has_server_for(path) {
                                let row = buf.cursor_row as u32;
                                let col = buf.cursor_col as u32;
                                self.lsp.request_completion(path, row, col);
                            }
                        }
                    }
                }
            }
            Command::InsertNewline => {
                if let Some(buf) = self.tabs.active_buffer_mut() {
                    buf.insert_newline();
                }
            }
            Command::DeleteBack => {
                if let Some(buf) = self.tabs.active_buffer_mut() {
                    buf.delete_back();
                }
            }
            Command::DeleteForward => {
                if let Some(buf) = self.tabs.active_buffer_mut() {
                    buf.delete_forward();
                }
            }
            Command::Undo => {
                if let Some(buf) = self.tabs.active_buffer_mut() {
                    buf.undo();
                }
            }

            // Folding
            Command::ToggleFold => {
                let line = self
                    .tabs
                    .active_buffer()
                    .map(|b| b.cursor_row)
                    .unwrap_or(0);
                if let Some(fold_state) = self.fold_states.get_mut(self.tabs.active) {
                    fold_state.toggle_fold(line);
                }
            }
            Command::FoldAll => {
                if let Some(fold_state) = self.fold_states.get_mut(self.tabs.active) {
                    let starts: Vec<usize> = fold_state.regions.iter().map(|r| r.start_line).collect();
                    for s in starts {
                        fold_state.fold(s);
                    }
                }
            }
            Command::UnfoldAll => {
                if let Some(fold_state) = self.fold_states.get_mut(self.tabs.active) {
                    fold_state.unfold_all();
                }
            }

            // File
            Command::Save => {
                if let Some(buf) = self.tabs.active_buffer_mut() {
                    let path = buf.file_path.clone();
                    match buf.save() {
                        Ok(()) => {
                            self.status_message = "Saved.".to_string();
                            if let Some(ref p) = path {
                                self.lsp.did_save(p);
                            }
                        }
                        Err(e) => self.status_message = format!("Save error: {}", e),
                    }
                }
            }
            Command::Quit => {
                if self.tabs.has_unsaved() {
                    self.status_message =
                        "Unsaved changes! Use Ctrl+Shift+Q to force quit.".to_string();
                } else {
                    self.running = false;
                }
            }
            Command::ForceQuit => {
                self.running = false;
            }

            // Tabs
            Command::NextTab => self.tabs.next_tab(),
            Command::PrevTab => self.tabs.prev_tab(),
            Command::CloseTab => {
                if let Some(buf) = self.tabs.active_buffer() {
                    if buf.dirty {
                        self.status_message = "Unsaved changes! Save first.".to_string();
                        return;
                    }
                }
                // Remove corresponding highlighters
                if self.tabs.active < self.highlighter_types.len() {
                    self.highlighter_types.remove(self.tabs.active);
                }
                if self.tabs.active < self.ts_highlighters.len() {
                    self.ts_highlighters.remove(self.tabs.active);
                }
                if self.tabs.active < self.fold_states.len() {
                    self.fold_states.remove(self.tabs.active);
                }
                self.tabs.close_tab();
            }

            // Explorer
            Command::ToggleExplorer => {
                self.explorer.visible = !self.explorer.visible;
                if self.explorer.visible && self.focus == FocusPane::Editor {
                    self.focus = FocusPane::Explorer;
                } else {
                    self.focus = FocusPane::Editor;
                }
            }
            Command::ExplorerUp => self.explorer.move_up(),
            Command::ExplorerDown => self.explorer.move_down(),
            Command::ExplorerEnter => {
                if let Some(path) = self.explorer.toggle_or_select() {
                    self.open_file(&path);
                }
            }

            // Focus
            Command::FocusExplorer => {
                if self.explorer.visible {
                    self.focus = FocusPane::Explorer;
                }
            }
            Command::FocusEditor => {
                self.focus = FocusPane::Editor;
            }

            // Search
            Command::SearchInFile => {
                self.search.active = true;
                self.search.across_files = false;
                self.search.query.clear();
                self.search.matches.clear();
                self.focus = FocusPane::SearchBar;
            }
            Command::SearchAcrossFiles => {
                self.search.active = true;
                self.search.across_files = true;
                self.search.query.clear();
                self.search.matches.clear();
                self.focus = FocusPane::SearchBar;
            }
            Command::SearchInput(c) => {
                self.search.query.push(c);
                self.perform_search();
            }
            Command::SearchBackspace => {
                self.search.query.pop();
                self.perform_search();
            }
            Command::SearchNext => {
                self.search.next_match();
                self.jump_to_match();
            }
            Command::SearchPrev => {
                self.search.prev_match();
                self.jump_to_match();
            }
            Command::SearchClose => {
                self.search.active = false;
                self.focus = FocusPane::Editor;
            }

            // Fuzzy finder
            Command::FuzzyOpen => {
                self.fuzzy.open();
                self.focus = FocusPane::FuzzyFinder;
            }
            Command::FuzzyInput(c) => {
                self.fuzzy.input(c);
            }
            Command::FuzzyBackspace => {
                self.fuzzy.backspace();
            }
            Command::FuzzyUp => {
                self.fuzzy.move_up();
            }
            Command::FuzzyDown => {
                self.fuzzy.move_down();
            }
            Command::FuzzyConfirm => {
                if let Some(path) = self.fuzzy.confirm() {
                    self.fuzzy.close();
                    self.open_file(&path);
                    self.focus = FocusPane::Editor;
                }
            }
            Command::FuzzyClose => {
                self.fuzzy.close();
                self.focus = FocusPane::Editor;
            }

            // Go to line
            Command::GoToLineOpen => {
                self.goto_line_input.clear();
                self.focus = FocusPane::GoToLine;
            }
            Command::GoToLineInput(c) => {
                self.goto_line_input.push(c);
            }
            Command::GoToLineBackspace => {
                self.goto_line_input.pop();
            }
            Command::GoToLineConfirm => {
                if let Ok(line_num) = self.goto_line_input.parse::<usize>() {
                    if let Some(buf) = self.tabs.active_buffer_mut() {
                        buf.go_to_line(line_num);
                    }
                }
                self.goto_line_input.clear();
                self.focus = FocusPane::Editor;
            }
            Command::GoToLineClose => {
                self.goto_line_input.clear();
                self.focus = FocusPane::Editor;
            }

            // LSP
            Command::LspHover => {
                if let Some(buf) = self.tabs.active_buffer() {
                    if let Some(ref path) = buf.file_path.clone() {
                        let row = buf.cursor_row as u32;
                        let col = buf.cursor_col as u32;
                        self.lsp.request_hover(path, row, col);
                    }
                }
            }
            Command::LspGotoDefinition => {
                if let Some(buf) = self.tabs.active_buffer() {
                    if let Some(ref path) = buf.file_path.clone() {
                        let row = buf.cursor_row as u32;
                        let col = buf.cursor_col as u32;
                        self.lsp.request_goto_definition(path, row, col);
                    }
                }
            }
            Command::LspComplete => {
                if let Some(buf) = self.tabs.active_buffer() {
                    if let Some(ref path) = buf.file_path.clone() {
                        let row = buf.cursor_row as u32;
                        let col = buf.cursor_col as u32;
                        self.lsp.request_completion(path, row, col);
                    }
                }
            }
            Command::CompletionUp => {
                if self.completion_active && !self.completion_items.is_empty() {
                    if self.completion_index > 0 {
                        self.completion_index -= 1;
                    } else {
                        self.completion_index = self.completion_items.len() - 1;
                    }
                }
            }
            Command::CompletionDown => {
                if self.completion_active && !self.completion_items.is_empty() {
                    self.completion_index =
                        (self.completion_index + 1) % self.completion_items.len();
                }
            }
            Command::CompletionConfirm => {
                if self.completion_active {
                    if let Some(item) = self.completion_items.get(self.completion_index) {
                        let text = item
                            .insert_text
                            .as_deref()
                            .unwrap_or(&item.label);
                        if let Some(buf) = self.tabs.active_buffer_mut() {
                            for c in text.chars() {
                                buf.insert_char(c);
                            }
                        }
                    }
                    self.completion_active = false;
                    self.completion_items.clear();
                    self.focus = FocusPane::Editor;
                }
            }
            Command::CompletionClose => {
                self.completion_active = false;
                self.completion_items.clear();
                self.focus = FocusPane::Editor;
            }
        }
    }

    fn perform_search(&mut self) {
        if self.search.across_files {
            let dir = self.working_dir.clone();
            self.search.search_across_files(&dir);
        } else if let Some(buf) = self.tabs.active_buffer() {
            let lines: Vec<String> = (0..buf.line_count())
                .filter_map(|i| buf.get_line(i))
                .collect();
            self.search.search_in_buffer(&lines);
        }
        self.status_message = format!(
            "Found {} matches",
            self.search.matches.len()
        );
    }

    fn jump_to_match(&mut self) {
        if let Some(m) = self.search.current_match().cloned() {
            if let Some(ref file) = m.file {
                // Cross-file: open the file first
                self.open_file(file);
            }
            if let Some(buf) = self.tabs.active_buffer_mut() {
                buf.cursor_row = m.line;
                buf.cursor_col = m.col;
            }
        }
    }

    pub fn handle_key(&mut self, event: crossterm::event::KeyEvent) {
        let key = convert_key_event(event);
        let cmd = self.keymap.map_key(key, self.focus);
        self.handle_command(cmd);
    }

    pub fn tick_lsp(&mut self) {
        self.lsp.poll();

        // Handle hover result
        if let Some(ref info) = self.lsp.hover_info {
            if !info.is_empty() {
                self.hover_display = Some(info.clone());
            } else {
                self.hover_display = None;
            }
            self.lsp.hover_info = None;
        }

        // Handle goto definition result
        if let Some((file, line, col)) = self.lsp.pending_goto.take() {
            self.open_file(&file);
            if let Some(buf) = self.tabs.active_buffer_mut() {
                buf.cursor_row = line;
                buf.cursor_col = col;
            }
        }

        // Handle completion result
        if let Some(items) = self.lsp.pending_completions.take() {
            if !items.is_empty() {
                self.completion_items = items;
                self.completion_index = 0;
                self.completion_active = true;
                self.focus = FocusPane::Completion;
            }
        }

        // Send didChange for dirty buffers
        for tab in &self.tabs.tabs {
            if tab.dirty {
                if let Some(ref path) = tab.file_path {
                    if self.lsp.has_server_for(path) {
                        let version = self
                            .doc_versions
                            .entry(path.clone())
                            .or_insert(1);
                        *version += 1;
                        let v = *version;
                        let text = tab.rope.to_string();
                        self.lsp.did_change(path, v, &text);
                    }
                }
            }
        }
    }

    pub fn tick_autosave(&mut self) {
        if self.settings.autosave_interval_secs == 0 {
            return;
        }
        let elapsed = self.last_autosave.elapsed().as_secs();
        if elapsed >= self.settings.autosave_interval_secs {
            let mut saved = Vec::new();
            for tab in &mut self.tabs.tabs {
                if tab.dirty && tab.file_path.is_some() {
                    if tab.save().is_ok() {
                        saved.push(tab.file_name());
                    }
                }
            }
            if !saved.is_empty() {
                self.status_message = format!("Autosaved: {}", saved.join(", "));
            }
            self.last_autosave = Instant::now();
        }
    }
}

fn convert_key_event(event: crossterm::event::KeyEvent) -> keymap::KeyEvent {
    use crossterm::event::{KeyCode as CK, KeyModifiers};

    let code = match event.code {
        CK::Char(c) => keymap::KeyCode::Char(c),
        CK::Enter => keymap::KeyCode::Enter,
        CK::Backspace => keymap::KeyCode::Backspace,
        CK::Delete => keymap::KeyCode::Delete,
        CK::Left => keymap::KeyCode::Left,
        CK::Right => keymap::KeyCode::Right,
        CK::Up => keymap::KeyCode::Up,
        CK::Down => keymap::KeyCode::Down,
        CK::Home => keymap::KeyCode::Home,
        CK::End => keymap::KeyCode::End,
        CK::PageUp => keymap::KeyCode::PageUp,
        CK::PageDown => keymap::KeyCode::PageDown,
        CK::Tab => keymap::KeyCode::Tab,
        CK::Esc => keymap::KeyCode::Esc,
        _ => keymap::KeyCode::Char('\0'),
    };

    let modifiers = keymap::Modifiers {
        ctrl: event.modifiers.contains(KeyModifiers::CONTROL),
        shift: event.modifiers.contains(KeyModifiers::SHIFT),
        alt: event.modifiers.contains(KeyModifiers::ALT),
        super_key: event.modifiers.contains(KeyModifiers::SUPER),
    };

    keymap::KeyEvent { code, modifiers }
}
