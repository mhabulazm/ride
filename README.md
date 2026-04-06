# RIDE — Rust IDE

A minimalist, fast, terminal-based IDE built in Rust.

## Supported File Types

| Extension | Language | Highlighting |
|-----------|----------|-------------|
| `.rs` | Rust | tree-sitter |
| `.py` | Python | tree-sitter |
| `.ts`, `.tsx` | TypeScript | tree-sitter |
| `.js`, `.jsx` | JavaScript | tree-sitter |
| `.go` | Go | tree-sitter |
| `.c`, `.h` | C | tree-sitter |
| `.cpp`, `.cc`, `.hpp`, `.cxx`, `.hxx` | C++ | tree-sitter |
| `.java` | Java | tree-sitter |
| `.md` | Markdown | tree-sitter |
| `.kt` | Kotlin | regex |
| `.proto` | Protocol Buffers | regex |
| `.log` | Log files | regex (level-aware) |
| `.mmd` | Mermaid diagrams | regex |
| `.txt` | Plain text | none |

## Installation

```bash
cargo build --release
```

The binary is at `target/release/ride`.

## Usage

```bash
# Open a directory
ride ./project

# Open a single file
ride ./src/App.java
```

## Keybindings

| Key | Action |
|-----|--------|
| Ctrl+Z | Undo |
| Ctrl+S | Save |
| Ctrl+Q | Quit |
| Ctrl+B | Toggle file explorer |
| Ctrl+W | Close tab |
| Ctrl+P | Fuzzy file finder |
| Ctrl+G | Go to line |
| Ctrl+H | LSP hover info |
| Ctrl+D | LSP go to definition |
| Ctrl+Space | LSP autocomplete |
| Ctrl+[ | Toggle fold at cursor |
| Ctrl+] | Unfold all |
| Ctrl+Left/Right | Word-wise cursor movement |
| Ctrl+PageDown / Alt+Right | Next tab |
| Ctrl+PageUp / Alt+Left | Previous tab |
| Ctrl+F | Search in file |
| Ctrl+Shift+F | Search across files |
| Ctrl+Home/End | Go to file start/end |
| Arrow keys | Move cursor |
| Home/End | Start/end of line |
| Page Up/Down | Scroll |
| Enter | New line (with auto-indent) |
| Esc | Close search / fuzzy finder / back to editor |
| Tab (in explorer) | Switch focus to editor |

All keybindings are shown on the welcome screen when no file is open. Keybindings are configurable via `keybindings.json` in the working directory.

## Features

- Tabbed editing with multiple open files
- File explorer with directory expand/collapse
- Fuzzy file finder (Ctrl+P) with scoring for consecutive and word-boundary matches
- Go-to-line dialog (Ctrl+G)
- Word-wise cursor movement (Ctrl+Left/Right)
- Auto-indent on newline (carries over leading whitespace)
- Bracket matching and highlighting for `()`, `{}`, `[]`
- Soft line wrapping (long lines wrap visually instead of horizontal scrolling)
- Code folding based on tree-sitter syntax tree (functions, classes, blocks, comments)
- Scope-aware syntax highlighting (method names, types, variables, annotations classified by context)
- In-file and cross-file search (case-insensitive)
- Syntax highlighting via tree-sitter (Rust, Python, TypeScript, JavaScript, Go, C, C++, Java, Markdown) with regex fallback (Kotlin, Protobuf, LOG, Mermaid)
- Configurable keybindings via JSON
- Configurable autosave (default: 5 minutes, set via `settings.json`)
- Large file support (streaming read/write via ropey)
- LSP client with diagnostics, hover, go-to-definition, and autocomplete
- Diagnostics gutter with severity indicators (● error, ▲ warning, ℹ info) and underline on affected ranges
- LSP autocomplete with popup menu (Ctrl+Space or auto-triggered on `.`/`:`)
- Configurable color themes (dark, light, monokai, solarized-dark) with custom overrides
- Welcome screen with keybinding reference
- Undo support

## Configuration

RIDE reads two JSON files from the working directory:

- `keybindings.json` — custom key bindings (see included file for format)
- `settings.json` — editor settings:

```json
{
  "autosave_interval_secs": 300,
  "theme": "monokai",
  "lsp": {
    "rs": { "command": "rust-analyzer", "args": [] },
    "py": { "command": "pylsp", "args": [] },
    "ts": { "command": "typescript-language-server", "args": ["--stdio"] }
  }
}
```

Set `autosave_interval_secs` to `0` to disable autosave. LSP servers are configured per file extension — the server is started on demand when a file of that type is opened.

### Themes

Built-in themes: `dark` (default), `light`, `monokai`, `solarized-dark`.

Set `"theme": "monokai"` to use a built-in theme. For custom overrides on top of a base theme:

```json
{
  "theme": {
    "base": "dark",
    "syntax": {
      "keyword": { "fg": "#ff79c6", "bold": true },
      "string": { "fg": "#f1fa8c" }
    },
    "ui": {
      "border_focused": "#bd93f9",
      "status_label": { "fg": "#282a36", "bg": "#50fa7b", "bold": true }
    }
  }
}
```

Colors can be named (`red`, `cyan`, `darkgray`) or hex (`#ff5733`).

## Architecture

```
ride/
├── crates/
│   ├── ride-core/   # Core library (UI-agnostic)
│   └── ride-tui/    # Terminal UI frontend (ratatui)
```

The core is decoupled from the UI, allowing a future GUI frontend (e.g. egui/iced) without rewriting editor logic.

## Tests

```bash
cargo test
```

125 unit tests covering buffer operations, auto-indent, word movement, bracket matching, code folding, tab management, keymap parsing and loading, search, fuzzy finder, settings, themes, and LSP message parsing.
