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
| Ctrl+PageDown / Alt+Right | Next tab |
| Ctrl+PageUp / Alt+Left | Previous tab |
| Ctrl+F | Search in file |
| Ctrl+Shift+F | Search across files |
| Ctrl+Home/End | Go to file start/end |
| Arrow keys | Move cursor |
| Home/End | Start/end of line |
| Page Up/Down | Scroll |
| Enter | New line / open file in explorer |
| Esc | Close search / back to editor |
| Tab (in explorer) | Switch focus to editor |

## Architecture

```
ride/
├── crates/
│   ├── ride-core/   # Core library (UI-agnostic)
│   └── ride-tui/    # Terminal UI frontend (ratatui)
```

The core is decoupled from the UI, allowing a future GUI frontend (e.g. egui/iced) without rewriting editor logic.
