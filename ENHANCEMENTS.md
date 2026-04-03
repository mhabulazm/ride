# RIDE Enhancement Ideas

## Editing Fundamentals

- Text selection and clipboard (copy/cut/paste)
- Multi-cursor editing
- Auto-indent on newline (carry over indentation from previous line)
- Find and replace (search exists but no replace)

## Tree-Sitter Integration

- Implement per-line caching so Java and Markdown use their tree-sitter parsers instead of the regex fallback
- Code folding based on syntax tree
- Scope-aware features and improved semantic highlighting

## File Handling

- File creation, rename, and delete from the explorer
- Non-UTF-8 encoding support

## Navigation and UX

- Minimap or scrollbar position indicator
- Mouse support (crossterm supports it, nothing is wired up)
- Soft line wrapping for long lines

## Quality of Life

- Configurable theme and colors (currently hardcoded in ui_editor.rs)
- Tab size and spaces-vs-tabs setting
- File type indicator in the status bar
- Multiple split panes (vertical/horizontal editor splits)
- Redo support (undo exists but no redo stack)
- Session restore (remember open tabs across runs)

## Language Support

- Additional tree-sitter grammars (Rust, Python, TypeScript, Go, etc.)
- LSP autocomplete / code actions
- Linting and diagnostics gutter (inline markers beyond line numbers)

## Infrastructure

- CI pipeline
- Plugin system for extensibility beyond keybindings
