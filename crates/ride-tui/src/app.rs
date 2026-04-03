use ride_core::command::{Command, FocusPane};
use ride_core::explorer::Explorer;
use ride_core::highlight::{self, HighlighterType};
use ride_core::keymap;
use ride_core::search::SearchState;
use ride_core::tab::TabManager;
use std::path::{Path, PathBuf};

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
                self.highlighter_types[self.tabs.active] = hl_type;
                self.focus = FocusPane::Editor;
                self.status_message = format!("Opened {}", path.display());
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

            // File
            Command::Save => {
                if let Some(buf) = self.tabs.active_buffer_mut() {
                    match buf.save() {
                        Ok(()) => self.status_message = "Saved.".to_string(),
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
                // Remove corresponding highlighter
                if self.tabs.active < self.highlighter_types.len() {
                    self.highlighter_types.remove(self.tabs.active);
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
        let cmd = keymap::map_key(key, self.focus);
        self.handle_command(cmd);
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
    };

    keymap::KeyEvent { code, modifiers }
}
