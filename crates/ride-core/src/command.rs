use serde::Deserialize;

/// All commands the editor can execute.
/// The UI layer translates key events into these commands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    // Cursor movement
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    MoveToLineStart,
    MoveToLineEnd,
    MoveToFileStart,
    MoveToFileEnd,
    MoveWordLeft,
    MoveWordRight,
    PageUp,
    PageDown,

    // Editing
    InsertChar(char),
    InsertNewline,
    DeleteBack,
    DeleteForward,
    Undo,

    // File operations
    Save,
    Quit,
    ForceQuit,

    // Tabs
    NextTab,
    PrevTab,
    CloseTab,

    // Explorer
    ToggleExplorer,
    ExplorerUp,
    ExplorerDown,
    ExplorerEnter,

    // Search
    SearchInFile,
    SearchAcrossFiles,
    SearchNext,
    SearchPrev,
    SearchClose,
    SearchInput(char),
    SearchBackspace,

    // Focus
    FocusExplorer,
    FocusEditor,

    // LSP
    LspHover,
    LspGotoDefinition,

    // Go to line
    GoToLineOpen,
    GoToLineInput(char),
    GoToLineBackspace,
    GoToLineConfirm,
    GoToLineClose,

    // Fuzzy finder
    FuzzyOpen,
    FuzzyInput(char),
    FuzzyBackspace,
    FuzzyUp,
    FuzzyDown,
    FuzzyConfirm,
    FuzzyClose,

    // No-op
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusPane {
    Editor,
    Explorer,
    SearchBar,
    FuzzyFinder,
    GoToLine,
}

/// Bindable commands (no data payload). Used for JSON keybinding deserialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum SimpleCommand {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    MoveToLineStart,
    MoveToLineEnd,
    MoveToFileStart,
    MoveToFileEnd,
    PageUp,
    PageDown,
    InsertNewline,
    DeleteBack,
    DeleteForward,
    Undo,
    Save,
    Quit,
    ForceQuit,
    NextTab,
    PrevTab,
    CloseTab,
    ToggleExplorer,
    ExplorerUp,
    ExplorerDown,
    ExplorerEnter,
    SearchInFile,
    SearchAcrossFiles,
    SearchNext,
    SearchPrev,
    SearchClose,
    SearchBackspace,
    FocusExplorer,
    FocusEditor,
    MoveWordLeft,
    MoveWordRight,
    LspHover,
    LspGotoDefinition,
    GoToLineOpen,
    GoToLineBackspace,
    GoToLineConfirm,
    GoToLineClose,
    FuzzyOpen,
    FuzzyUp,
    FuzzyDown,
    FuzzyConfirm,
    FuzzyClose,
    FuzzyBackspace,
}

impl SimpleCommand {
    pub fn into_command(self) -> Command {
        match self {
            Self::MoveUp => Command::MoveUp,
            Self::MoveDown => Command::MoveDown,
            Self::MoveLeft => Command::MoveLeft,
            Self::MoveRight => Command::MoveRight,
            Self::MoveToLineStart => Command::MoveToLineStart,
            Self::MoveToLineEnd => Command::MoveToLineEnd,
            Self::MoveToFileStart => Command::MoveToFileStart,
            Self::MoveToFileEnd => Command::MoveToFileEnd,
            Self::PageUp => Command::PageUp,
            Self::PageDown => Command::PageDown,
            Self::InsertNewline => Command::InsertNewline,
            Self::DeleteBack => Command::DeleteBack,
            Self::DeleteForward => Command::DeleteForward,
            Self::Undo => Command::Undo,
            Self::Save => Command::Save,
            Self::Quit => Command::Quit,
            Self::ForceQuit => Command::ForceQuit,
            Self::NextTab => Command::NextTab,
            Self::PrevTab => Command::PrevTab,
            Self::CloseTab => Command::CloseTab,
            Self::ToggleExplorer => Command::ToggleExplorer,
            Self::ExplorerUp => Command::ExplorerUp,
            Self::ExplorerDown => Command::ExplorerDown,
            Self::ExplorerEnter => Command::ExplorerEnter,
            Self::SearchInFile => Command::SearchInFile,
            Self::SearchAcrossFiles => Command::SearchAcrossFiles,
            Self::SearchNext => Command::SearchNext,
            Self::SearchPrev => Command::SearchPrev,
            Self::SearchClose => Command::SearchClose,
            Self::SearchBackspace => Command::SearchBackspace,
            Self::FocusExplorer => Command::FocusExplorer,
            Self::FocusEditor => Command::FocusEditor,
            Self::MoveWordLeft => Command::MoveWordLeft,
            Self::MoveWordRight => Command::MoveWordRight,
            Self::LspHover => Command::LspHover,
            Self::LspGotoDefinition => Command::LspGotoDefinition,
            Self::GoToLineOpen => Command::GoToLineOpen,
            Self::GoToLineBackspace => Command::GoToLineBackspace,
            Self::GoToLineConfirm => Command::GoToLineConfirm,
            Self::GoToLineClose => Command::GoToLineClose,
            Self::FuzzyOpen => Command::FuzzyOpen,
            Self::FuzzyUp => Command::FuzzyUp,
            Self::FuzzyDown => Command::FuzzyDown,
            Self::FuzzyConfirm => Command::FuzzyConfirm,
            Self::FuzzyClose => Command::FuzzyClose,
            Self::FuzzyBackspace => Command::FuzzyBackspace,
        }
    }
}
