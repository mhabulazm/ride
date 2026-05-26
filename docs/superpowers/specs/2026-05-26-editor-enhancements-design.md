# Design: Markdown preview, HTML highlighting, git change markers, colorblind theme

Date: 2026-05-26
Status: Approved (pending spec review)

## Overview

Four independent enhancements to RIDE, plus one shared touch-point in the theme
system:

1. **HTML syntax highlighting** — add `.html`/`.htm` as a tree-sitter language.
2. **Markdown preview** — a full-window toggle that renders the active Markdown
   buffer as formatted text.
3. **Git change markers** — a gutter marker column plus a subtle line tint
   showing lines added/modified/removed versus `HEAD`.
4. **Colorblind theme** — a red-green-safe (Okabe-Ito) dark built-in theme.

Markdown is *already* syntax-highlighted via tree-sitter; this work adds a
rendered **preview**, not highlighting. HTML is *not* currently supported and
gets syntax highlighting only (no rendering).

## Shared touch-point: theme system

The git feature introduces three new color fields. These must land first (or
together with the git and colorblind work) because every theme must define them.

- Add to `ride-core/src/theme.rs` `UiColors`:
  - `git_added: ColorStyle`
  - `git_modified: ColorStyle`
  - `git_removed: ColorStyle`
- For each: `fg` = the gutter-marker color; optional `bg` = the subtle line
  tint. This yields the "marker + tint" behavior from a single field.
- Add the same three as `Option<ColorStyle>` to `UiOverride`, and add
  `apply_ui_cs!(git_added)` / `git_modified` / `git_removed` to `Theme::resolve`.
- Define the three fields in **all** built-in themes: `dark`, `light`,
  `monokai`, `solarized-dark`, and the new `colorblind`.
- Update the existing `test_builtin_names` test: it asserts `len == 4` and must
  become `5` once `colorblind` is registered.

Default git colors per existing theme (sensible per-palette values):
- dark: added `green`, modified `yellow`, removed `red`.
- light/monokai/solarized-dark: nearest palette equivalents.
- colorblind: added `#0072B2` (blue), modified `#E69F00` (orange), removed
  `#D55E00` (vermillion) — deliberately off the red-green axis.

## Feature 1 — HTML syntax highlighting

**Dependency:** add `tree-sitter-html` to `ride-core/Cargo.toml`, version from the
`0.23.x` line (compatible with the pinned `tree-sitter = "0.24"`, matching the
other `0.23` grammars already in the tree).

**`ride-core/src/highlight.rs`:**
- Add `Html` to `enum TreeSitterLang`.
- In `detect_highlighter`, map `Some("html" | "htm")` →
  `HighlighterType::TreeSitter(TreeSitterLang::Html)`.

**`ride-core/src/highlight/treesitter_hl.rs`:**
- `get_language`: `TreeSitterLang::Html => Some(tree_sitter_html::LANGUAGE.into())`.
- `lang_name`: `TreeSitterLang::Html => "html"`.
- `scope_aware_highlight`: dispatch `TreeSitterLang::Html => self.html_highlight(...)`.
- New `html_highlight(node_kind, parent_kind) -> HighlightKind`:
  - `comment` → `Comment`
  - `tag_name`, `erroneous_end_tag_name` → `Type`
  - `attribute_name` → `Variable`
  - `attribute_value`, `quoted_attribute_value`, `"` → `String`
  - `doctype` → `Keyword`
  - `<`, `>`, `</`, `/>`, `=` → `Punctuation`
  - `text`, default → `Normal`

**Known limitation:** embedded `<script>` / `<style>` content is not highlighted
(no tree-sitter language injections). Such content renders as `Normal`. This is
acceptable for the current scope.

**Folding:** `FoldState::update_regions_from_tree` is called with `lang_name`
`"html"`. HTML folding is not required; verify the existing folding code returns
no regions (rather than panicking) for an unrecognized language name. No HTML
folding rules are added in this work.

**Docs:** add an HTML row to the README "Supported Languages" table.

**Tests:**
- `detect_highlighter` maps `.html` and `.htm` to the HTML tree-sitter type.
- `html_highlight` classifies representative node kinds (tag_name, attribute_name,
  quoted_attribute_value, comment, punctuation).

## Feature 2 — Markdown preview (full-window toggle)

**Dependency:** add `pulldown-cmark` to `ride-core/Cargo.toml` (pure-Rust,
lightweight).

**`ride-core/src/preview.rs` (new, UI-agnostic):**
- `pub fn render_markdown(source: &str) -> Vec<PreviewLine>`
- `pub struct PreviewLine { pub spans: Vec<PreviewSpan> }`
- `pub struct PreviewSpan { pub text: String, pub style: PreviewStyle }`
- `pub enum PreviewStyle { Normal, Heading(u8), Bold, Italic, Code, Link,
  ListItem, BlockQuote, Rule }`
- No ratatui types here — fully unit-testable in `ride-core`.
- Driven by the `pulldown-cmark` event stream, folded into logical terminal
  lines. Rendering conventions (a terminal cannot resize fonts):
  - Headings → `Heading(level)` (UI renders bold + heading color, with a leading
    bar/`#` marker).
  - Strong → `Bold`; emphasis → `Italic`.
  - Inline code and fenced code blocks → `Code`.
  - Unordered lists → `• ` prefix; ordered lists → `N. ` prefix; nested lists
    indented.
  - Block quotes → `▌ ` prefix as `BlockQuote`.
  - Links → link text styled `Link`, followed by the URL dim in parentheses.
  - Thematic breaks → a `Rule` line rendered as `─` repeated.
  - Images → literal `[image: <alt>]` text.
- Long lines are left for the UI to wrap (ratatui handles wrapping).

**`ride-tui/src/ui_preview.rs` (new):**
- `pub fn render_preview(frame, area, app)` converts `Vec<PreviewLine>` to
  ratatui `Line`s, mapping `PreviewStyle` to **existing** theme fields:
  - `Heading` → `theme.syntax.heading`
  - `Link` → `theme.syntax.link`
  - `Code` → `theme.syntax.string`
  - `Italic`/`Bold` → `theme.syntax.emphasis` + bold modifier
  - `BlockQuote` → `theme.syntax.comment`
  - `Rule`/`ListItem`/`Normal` → sensible existing colors
- **No new theme fields** are required for the preview.
- Honors `app.preview_scroll` as the first visible line.

**App + command wiring:**
- `App` gains `pub preview_active: bool` (default false) and
  `pub preview_scroll: usize` (default 0).
- New `Command::TogglePreview`, `SimpleCommand::TogglePreview`, and the
  `into_command` arm.
- Default keybinding: `ctrl+e` in the `editor` context (verified unused).
- `handle_command`:
  - `TogglePreview`: if the active buffer's highlighter is
    `TreeSitterLang::Markdown`, flip `preview_active` and reset `preview_scroll`
    to 0; otherwise set a status message ("Preview is only available for
    Markdown files") and do nothing.
  - While `preview_active`, `MoveUp`/`MoveDown`/`PageUp`/`PageDown` adjust
    `preview_scroll` instead of moving the cursor. `TogglePreview` returns to
    editing. (No new `FocusPane`; the editor context is reused.)
- `ride-tui/src/ui.rs`: when `preview_active` and the active buffer is Markdown,
  render `render_preview` into the content area in place of `render_editor`.

**Docs:** add the Ctrl+E preview binding to the README keybindings table and the
welcome screen in `ui_editor.rs`.

**Tests:** `render_markdown` produces the expected `PreviewLine`/`PreviewStyle`
model for headings, bold, italic, unordered + ordered lists, fenced code blocks,
block quotes, links, and thematic breaks.

## Feature 3 — Git change markers + tint

**No new dependency.** Uses the `git` binary already on `PATH`.

**`ride-core/src/git.rs` (new):**
- `pub enum LineStatus { Unchanged, Added, Modified }`
- `pub struct GitLineDiff { pub status: Vec<LineStatus>, pub deleted_before:
  std::collections::HashSet<usize> }`
  - `status[i]` is the change state of current line `i`.
  - `deleted_before` holds indices of current lines that have one or more deleted
    lines immediately above them (drives the "removed" marker).
- `pub fn head_blob(working_dir: &Path, file_path: &Path) -> Option<String>`:
  - Resolve the repo (`git -C <working_dir> rev-parse --show-toplevel`) and the
    file path relative to the repo root.
  - Run `git -C <root> show HEAD:<relpath>`; return `Some(text)` on success.
  - Return `None` when: not a git repo, no `HEAD` (empty repo), or the path is
    untracked / not in `HEAD`.
- `pub fn diff_lines(head: &str, current: &str) -> GitLineDiff`:
  - A small in-process line-based diff (LCS / Myers). Lines only in `current` →
    `Added`; a delete+insert aligned at the same position → `Modified`; pure
    deletions → record the following current line index (or the last line) in
    `deleted_before`. This pure function is the primary unit-test target and
    needs no git repo.
- **Untracked-but-in-repo** files (where `head_blob` is `None` but the working
  dir *is* a repo): treat every line as `Added`. Distinguish "in a repo" from
  "not a repo" so non-repo files show nothing.

**App wiring:**
- `App` gains per-tab `pub git_baselines: Vec<Option<String>>` (the `HEAD` blob
  text), grown in lockstep with the other per-tab vecs in `open_file`.
- Populate `git_baselines[active]` via `git::head_blob` on `open_file` and
  refresh it on `Save`.
- A helper `App::active_git_diff(&self) -> Option<GitLineDiff>` computes
  `diff_lines(baseline, current_buffer_text)` for the active tab. Recomputed when
  the buffer is dirty so markers update **as you type**; for typical file sizes
  recomputation per frame is acceptable (cache keyed on a dirty/version check if
  profiling shows a need).

**`ride-tui/src/ui_editor.rs`:**
- Add a **1-character git marker column** to the gutter, before the diagnostic
  symbol. `gutter_width` goes from 6 to 7; the blank-continuation gutter and the
  tilde-fill widths update accordingly.
- Marker glyphs (colored with the change type's `fg`):
  - `Added` / `Modified` → `│` in `git_added` / `git_modified`.
  - A line in `deleted_before` → `▁` (or `_`) in `git_removed`.
  - `Unchanged` → space.
- **Tint:** when the change type's `ColorStyle` has a `bg`, apply that `bg`
  across the line's `style_map` (and the rendered chunk), producing the subtle
  full-line tint alongside the marker.
- When not in a git repo (no baseline and not a repo), render the column blank
  and apply no tint.

**Status bar (included):** show `+a ~m -d` change counts for the active buffer in
`ui_status.rs`, derived from `active_git_diff`.

**Tests:** `diff_lines` unit tests for added-only, modified, removed-only, mixed,
and unchanged cases (no git required). Optionally, a `head_blob` integration test
using a `tempfile` git repo (the `tempfile` dev-dependency already exists).

## Feature 4 — Colorblind theme (red-green safe, dark)

**`ride-core/src/theme.rs`:**
- `pub fn colorblind_dark_theme() -> Theme`, name `"colorblind"`.
- Register `"colorblind"` in `Theme::builtin` and `Theme::builtin_names`.
- Palette: Okabe-Ito on a dark background. The critical wins are that
  diagnostics and git colors avoid the red-green axis:
  - Syntax: keyword `#E69F00` (orange, bold), type `#56B4E9` (sky blue),
    function `#0072B2` (blue), string `#009E73` (bluish green), number
    `#CC79A7` (reddish purple), comment `#999999` (gray, italic), operator
    `#D55E00` (vermillion), punctuation/variable light gray, heading sky-blue
    bold, link sky-blue underline, emphasis italic.
  - Diagnostics: error `#D55E00` (vermillion, bold) with the `●` symbol,
    warning `#E69F00` (orange) with `▲`, info `#56B4E9` (sky blue), hint gray.
    Hue + symbol disambiguate error vs warning without red-green reliance.
  - Git: added `#0072B2` (blue), modified `#E69F00` (orange), removed `#D55E00`
    (vermillion).
  - UI chrome (borders, status bar, tabs, explorer, fuzzy, etc.): drawn from the
    same palette, consistent with the structure of the other theme constructors.

**Docs:** add `colorblind` to the README built-in theme list, and document the
`git_added` / `git_modified` / `git_removed` theme override fields.

**Tests:** `Theme::builtin("colorblind").is_some()`; `builtin_names` contains
`"colorblind"` and the length assertion updates from 4 to 5.

## Suggested build order

Each step is independently shippable as its own commit:

1. **Theme git fields** — add `git_added`/`git_modified`/`git_removed` to
   `UiColors`, `UiOverride`, `resolve`, and all existing themes. (Prerequisite
   for steps 2 and 3.)
2. **Colorblind theme** — add and register `colorblind_dark_theme`.
3. **Git markers + tint** — `git.rs`, app wiring, gutter rendering, status counts.
4. **HTML highlighting** — grammar dep, detection, `html_highlight`.
5. **Markdown preview** — `pulldown-cmark` dep, `preview.rs`, `ui_preview.rs`,
   command + keybinding.

## Out of scope

- Rendering HTML as a web page (terminal HTML rendering); HTML is highlight-only.
- Tree-sitter language injections (embedded CSS/JS highlighting inside HTML).
- Git staging, blame, commit, or any write operations; this is display-only,
  comparing the working buffer against `HEAD`.
- Watching `.git` for external HEAD changes; the baseline refreshes on open and
  on save.
- Light or additional colorblind variants beyond the single dark theme.
