# RIDE Enhancement Ideas

## Recently Added

- HTML syntax highlighting (`.html`, `.htm`) via tree-sitter
- Markdown preview toggle (Ctrl+E)
- Git change markers in the gutter with live diff against HEAD (`+a ~m -d` counts in status bar)
- `colorblind` built-in theme (red-green-safe, Okabe-Ito palette)

## Editing Fundamentals

- Text selection and clipboard (copy/cut/paste)
- Multi-cursor editing
- Find and replace (search exists but no replace)

## File Handling

- Non-UTF-8 encoding support

## Navigation and UX

- Minimap or scrollbar position indicator
- Mouse support (crossterm supports it, nothing is wired up)

## Quality of Life

- Tab size and spaces-vs-tabs setting
- File type indicator in the status bar
- Multiple split panes (vertical/horizontal editor splits)
- Redo support (undo exists but no redo stack)
- Session restore (remember open tabs across runs)

## Language Support

- LSP rename symbol
- LSP signature help

## Infrastructure

- Plugin system for extensibility beyond keybindings
- GUI frontend (egui/iced)
