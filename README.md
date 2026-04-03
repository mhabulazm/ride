# RIDE — Rust IDE

A minimalist, fast, terminal-based IDE built in Rust.

## Supported File Types

- `.java` — Java (tree-sitter highlighting)
- `.kt` — Kotlin (regex highlighting)
- `.md` — Markdown (tree-sitter highlighting)
- `.proto` — Protocol Buffers (regex highlighting)
- `.LOG` — Log files (level-aware coloring)
- `.mmd` — Mermaid diagrams (keyword highlighting)
- `.txt` — Plain text

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
| Ctrl+Left/Right | Word-wise cursor movement |
| Ctrl+PageDown / Alt+Right | Next tab |
| Ctrl+PageUp / Alt+Left | Previous tab |
| Ctrl+F | Search in file |
| Ctrl+Shift+F | Search across files |
| Ctrl+Home/End | Go to file start/end |
| Arrow keys | Move cursor |
| Home/End | Start/end of line |
| Page Up/Down | Scroll |
| Enter | New line / open file in explorer |
| Esc | Close search / fuzzy finder / back to editor |
| Tab (in explorer) | Switch focus to editor |

Keybindings are configurable via `keybindings.json` in the working directory. See the included file for the default bindings and format.

## Features

- Tabbed editing with multiple open files
- File explorer with directory expand/collapse
- Fuzzy file finder (Ctrl+P) with scoring for consecutive and word-boundary matches
- Go-to-line dialog (Ctrl+G)
- Word-wise cursor movement (Ctrl+Left/Right)
- Bracket matching and highlighting for `()`, `{}`, `[]`
- In-file and cross-file search (case-insensitive)
- Syntax highlighting via tree-sitter (Java, Markdown) and regex (Kotlin, Protobuf, LOG, Mermaid)
- Configurable keybindings via JSON
- Configurable autosave (default: 5 minutes, set via `settings.json`)
- Large file support (streaming read/write via ropey)
- LSP client with diagnostics, hover, and go-to-definition
- Undo support

## Configuration

RIDE reads two JSON files from the working directory:

- `keybindings.json` — custom key bindings (see included file for format)
- `settings.json` — editor settings:

```json
{
  "autosave_interval_secs": 300,
  "lsp": {
    "rs": { "command": "rust-analyzer", "args": [] },
    "py": { "command": "pylsp", "args": [] },
    "ts": { "command": "typescript-language-server", "args": ["--stdio"] }
  }
}
```

Set `autosave_interval_secs` to `0` to disable autosave. LSP servers are configured per file extension — the server is started on demand when a file of that type is opened.

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

104 unit tests covering buffer operations, word movement, bracket matching, tab management, keymap parsing and loading, search, fuzzy finder, settings, and LSP message parsing.
