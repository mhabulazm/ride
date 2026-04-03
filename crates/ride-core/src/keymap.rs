use crate::command::{Command, FocusPane};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub modifiers: Modifiers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

pub fn map_key(event: KeyEvent, focus: FocusPane) -> Command {
    match focus {
        FocusPane::SearchBar => map_search_key(event),
        FocusPane::Explorer => map_explorer_key(event),
        FocusPane::Editor => map_editor_key(event),
    }
}

fn map_editor_key(event: KeyEvent) -> Command {
    match (event.code, event.modifiers) {
        // Ctrl shortcuts
        (KeyCode::Char('s'), m) if m.ctrl => Command::Save,
        (KeyCode::Char('q'), m) if m.ctrl => Command::Quit,
        (KeyCode::Char('w'), m) if m.ctrl => Command::CloseTab,
        (KeyCode::Char('b'), m) if m.ctrl => Command::ToggleExplorer,
        (KeyCode::Char('f'), m) if m.ctrl && !m.shift => Command::SearchInFile,
        (KeyCode::Char('z'), m) if m.ctrl => Command::Undo,
        (KeyCode::Char('f'), m) if m.ctrl && m.shift => Command::SearchAcrossFiles,
        (KeyCode::PageDown, m) if m.ctrl => Command::NextTab,
        (KeyCode::PageUp, m) if m.ctrl => Command::PrevTab,
        (KeyCode::Left, m) if m.alt => Command::PrevTab,
        (KeyCode::Right, m) if m.alt => Command::NextTab,
        (KeyCode::Home, m) if m.ctrl => Command::MoveToFileStart,
        (KeyCode::End, m) if m.ctrl => Command::MoveToFileEnd,

        // Navigation
        (KeyCode::Up, _) => Command::MoveUp,
        (KeyCode::Down, _) => Command::MoveDown,
        (KeyCode::Left, _) => Command::MoveLeft,
        (KeyCode::Right, _) => Command::MoveRight,
        (KeyCode::Home, _) => Command::MoveToLineStart,
        (KeyCode::End, _) => Command::MoveToLineEnd,
        (KeyCode::PageUp, _) => Command::PageUp,
        (KeyCode::PageDown, _) => Command::PageDown,

        // Editing
        (KeyCode::Enter, _) => Command::InsertNewline,
        (KeyCode::Backspace, _) => Command::DeleteBack,
        (KeyCode::Delete, _) => Command::DeleteForward,
        (KeyCode::Char(c), m) if !m.ctrl && !m.alt => Command::InsertChar(c),
        (KeyCode::Tab, _) => Command::InsertChar('\t'),

        _ => Command::None,
    }
}

fn map_explorer_key(event: KeyEvent) -> Command {
    match (event.code, event.modifiers) {
        (KeyCode::Char('q'), m) if m.ctrl => Command::Quit,
        (KeyCode::Char('b'), m) if m.ctrl => Command::ToggleExplorer,
        (KeyCode::Char('f'), m) if m.ctrl && !m.shift => Command::SearchInFile,
        (KeyCode::Char('f'), m) if m.ctrl && m.shift => Command::SearchAcrossFiles,
        (KeyCode::Up, _) => Command::ExplorerUp,
        (KeyCode::Down, _) => Command::ExplorerDown,
        (KeyCode::Enter, _) => Command::ExplorerEnter,
        (KeyCode::Tab, _) => Command::FocusEditor,
        (KeyCode::Esc, _) => Command::FocusEditor,
        _ => Command::None,
    }
}

fn map_search_key(event: KeyEvent) -> Command {
    match (event.code, event.modifiers) {
        (KeyCode::Esc, _) => Command::SearchClose,
        (KeyCode::Enter, _) => Command::SearchNext,
        (KeyCode::Char('n'), m) if m.ctrl => Command::SearchNext,
        (KeyCode::Char('p'), m) if m.ctrl => Command::SearchPrev,
        (KeyCode::Backspace, _) => Command::SearchBackspace,
        (KeyCode::Char(c), m) if !m.ctrl && !m.alt => Command::SearchInput(c),
        _ => Command::None,
    }
}
