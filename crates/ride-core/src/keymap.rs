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
}

impl Modifiers {
    pub fn none() -> Self {
        Self {
            ctrl: false,
            shift: false,
            alt: false,
        }
    }

    pub fn ctrl() -> Self {
        Self {
            ctrl: true,
            shift: false,
            alt: false,
        }
    }

    pub fn ctrl_shift() -> Self {
        Self {
            ctrl: true,
            shift: true,
            alt: false,
        }
    }

    pub fn alt() -> Self {
        Self {
            ctrl: false,
            shift: false,
            alt: true,
        }
    }
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
}

impl KeymapConfig {
    /// Load keybindings from a JSON file, falling back to defaults if the file
    /// is missing or malformed.
    pub fn load(path: &Path) -> Self {
        let file_path = path.join("keybindings.json");
        match std::fs::read_to_string(&file_path) {
            Ok(contents) => match serde_json::from_str::<KeybindingsFile>(&contents) {
                Ok(file) => Self::from_file(&file),
                Err(_) => Self::defaults(),
            },
            Err(_) => Self::defaults(),
        }
    }

    /// Build from the JSON file, using defaults as the base and overriding
    /// with any bindings defined in the file.
    fn from_file(file: &KeybindingsFile) -> Self {
        let mut config = Self::defaults();

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

        config
    }

    /// Hardcoded defaults matching the original keybindings.
    pub fn defaults() -> Self {
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
        editor.insert(key('f', Modifiers::ctrl_shift()), Command::SearchAcrossFiles);
        editor.insert(
            KeyEvent { code: KeyCode::PageDown, modifiers: Modifiers::ctrl() },
            Command::NextTab,
        );
        editor.insert(
            KeyEvent { code: KeyCode::PageUp, modifiers: Modifiers::ctrl() },
            Command::PrevTab,
        );
        editor.insert(
            KeyEvent { code: KeyCode::Left, modifiers: Modifiers::alt() },
            Command::PrevTab,
        );
        editor.insert(
            KeyEvent { code: KeyCode::Right, modifiers: Modifiers::alt() },
            Command::NextTab,
        );
        editor.insert(
            KeyEvent { code: KeyCode::Home, modifiers: Modifiers::ctrl() },
            Command::MoveToFileStart,
        );
        editor.insert(
            KeyEvent { code: KeyCode::End, modifiers: Modifiers::ctrl() },
            Command::MoveToFileEnd,
        );
        editor.insert(special(KeyCode::Up), Command::MoveUp);
        editor.insert(special(KeyCode::Down), Command::MoveDown);
        editor.insert(special(KeyCode::Left), Command::MoveLeft);
        editor.insert(special(KeyCode::Right), Command::MoveRight);
        editor.insert(
            KeyEvent { code: KeyCode::Left, modifiers: Modifiers::ctrl() },
            Command::MoveWordLeft,
        );
        editor.insert(
            KeyEvent { code: KeyCode::Right, modifiers: Modifiers::ctrl() },
            Command::MoveWordRight,
        );
        editor.insert(key('g', Modifiers::ctrl()), Command::GoToLineOpen);
        editor.insert(key('h', Modifiers::ctrl()), Command::LspHover);
        editor.insert(key('d', Modifiers::ctrl()), Command::LspGotoDefinition);
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
        explorer.insert(key('f', Modifiers::ctrl_shift()), Command::SearchAcrossFiles);
        explorer.insert(special(KeyCode::Up), Command::ExplorerUp);
        explorer.insert(special(KeyCode::Down), Command::ExplorerDown);
        explorer.insert(special(KeyCode::Enter), Command::ExplorerEnter);
        explorer.insert(special(KeyCode::Tab), Command::FocusEditor);
        explorer.insert(special(KeyCode::Esc), Command::FocusEditor);

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

        Self { editor, explorer, search, fuzzy, goto_line }
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
        };

        if let Some(cmd) = table.get(&event) {
            return cmd.clone();
        }

        // Hardcoded character fallbacks
        match focus {
            FocusPane::Editor => {
                if let KeyCode::Char(c) = event.code {
                    if !event.modifiers.ctrl && !event.modifiers.alt {
                        return Command::InsertChar(c);
                    }
                }
            }
            FocusPane::SearchBar => {
                if let KeyCode::Char(c) = event.code {
                    if !event.modifiers.ctrl && !event.modifiers.alt {
                        return Command::SearchInput(c);
                    }
                }
            }
            FocusPane::FuzzyFinder => {
                if let KeyCode::Char(c) = event.code {
                    if !event.modifiers.ctrl && !event.modifiers.alt {
                        return Command::FuzzyInput(c);
                    }
                }
            }
            FocusPane::GoToLine => {
                if let KeyCode::Char(c) = event.code {
                    if !event.modifiers.ctrl && !event.modifiers.alt && c.is_ascii_digit() {
                        return Command::GoToLineInput(c);
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
    KeyEvent { code: KeyCode::Char(ch), modifiers }
}

fn special(code: KeyCode) -> KeyEvent {
    KeyEvent { code, modifiers: Modifiers::none() }
}

/// Parse a key string like "ctrl+shift+f" or "pagedown" into a KeyEvent.
fn parse_key_string(s: &str) -> Option<KeyEvent> {
    let parts: Vec<&str> = s.split('+').collect();
    let mut ctrl = false;
    let mut shift = false;
    let mut alt = false;

    // All parts except the last are modifiers
    for &part in &parts[..parts.len() - 1] {
        match part.to_lowercase().as_str() {
            "ctrl" => ctrl = true,
            "shift" => shift = true,
            "alt" => alt = true,
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
        modifiers: Modifiers { ctrl, shift, alt },
    })
}

// Keep the free function for backward compatibility during transition
pub fn map_key(event: KeyEvent, focus: FocusPane) -> Command {
    KeymapConfig::defaults().map_key(event, focus)
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
        let config = KeymapConfig::defaults();
        let event = KeyEvent {
            code: KeyCode::Char('s'),
            modifiers: Modifiers::ctrl(),
        };
        assert_eq!(config.map_key(event, FocusPane::Editor), Command::Save);
    }

    #[test]
    fn test_defaults_editor_arrow_keys() {
        let config = KeymapConfig::defaults();
        let up = KeyEvent { code: KeyCode::Up, modifiers: Modifiers::none() };
        assert_eq!(config.map_key(up, FocusPane::Editor), Command::MoveUp);
    }

    #[test]
    fn test_editor_char_fallback() {
        let config = KeymapConfig::defaults();
        let event = KeyEvent {
            code: KeyCode::Char('x'),
            modifiers: Modifiers::none(),
        };
        assert_eq!(config.map_key(event, FocusPane::Editor), Command::InsertChar('x'));
    }

    #[test]
    fn test_search_char_fallback() {
        let config = KeymapConfig::defaults();
        let event = KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: Modifiers::none(),
        };
        assert_eq!(config.map_key(event, FocusPane::SearchBar), Command::SearchInput('a'));
    }

    #[test]
    fn test_fuzzy_char_fallback() {
        let config = KeymapConfig::defaults();
        let event = KeyEvent {
            code: KeyCode::Char('m'),
            modifiers: Modifiers::none(),
        };
        assert_eq!(config.map_key(event, FocusPane::FuzzyFinder), Command::FuzzyInput('m'));
    }

    #[test]
    fn test_explorer_enter() {
        let config = KeymapConfig::defaults();
        let event = KeyEvent { code: KeyCode::Enter, modifiers: Modifiers::none() };
        assert_eq!(config.map_key(event, FocusPane::Explorer), Command::ExplorerEnter);
    }

    #[test]
    fn test_unbound_key_returns_none() {
        let config = KeymapConfig::defaults();
        let event = KeyEvent {
            code: KeyCode::Char('x'),
            modifiers: Modifiers { ctrl: true, shift: true, alt: true },
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
        let config = KeymapConfig::load(dir.path());

        // Custom binding works
        let event = KeyEvent { code: KeyCode::Char('k'), modifiers: Modifiers::ctrl() };
        assert_eq!(config.map_key(event, FocusPane::Editor), Command::Save);

        // Default binding still works
        let event = KeyEvent { code: KeyCode::Up, modifiers: Modifiers::none() };
        assert_eq!(config.map_key(event, FocusPane::Editor), Command::MoveUp);
    }

    #[test]
    fn test_load_missing_file_uses_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let config = KeymapConfig::load(dir.path());
        let event = KeyEvent { code: KeyCode::Char('s'), modifiers: Modifiers::ctrl() };
        assert_eq!(config.map_key(event, FocusPane::Editor), Command::Save);
    }

    #[test]
    fn test_load_malformed_json_uses_defaults() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("keybindings.json"), "not json!").unwrap();
        let config = KeymapConfig::load(dir.path());
        let event = KeyEvent { code: KeyCode::Char('s'), modifiers: Modifiers::ctrl() };
        assert_eq!(config.map_key(event, FocusPane::Editor), Command::Save);
    }
}
