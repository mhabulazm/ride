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

    // No-op
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusPane {
    Editor,
    Explorer,
    SearchBar,
}
