# Editor Enhancements Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add HTML syntax highlighting, a Markdown preview toggle, git change markers (marker + line tint), and a colorblind-safe dark theme to RIDE.

**Architecture:** Pure, UI-agnostic logic lives in `ride-core` (theme data, a line-diff function, a Markdown render model); the `ride-tui` crate renders that data with ratatui. Git baselines come from shelling out to `git show HEAD:<path>` and are diffed in-process against the live buffer. Each feature is an independent, separately-committable slice.

**Tech Stack:** Rust (workspace: `ride-core` lib + `ride-tui` ratatui frontend), tree-sitter grammars, `pulldown-cmark` (new), the system `git` binary.

---

## File Structure

**Created:**
- `crates/ride-core/src/git.rs` — line-diff core (`diff_lines`) + git baseline access (`head_blob`, `is_repo`). One responsibility: "what changed vs HEAD".
- `crates/ride-core/src/preview.rs` — Markdown → UI-agnostic render model (`render_markdown`). One responsibility: "turn Markdown text into styled lines".
- `crates/ride-tui/src/ui_preview.rs` — render a `Vec<PreviewLine>` with ratatui + theme.

**Modified:**
- `crates/ride-core/Cargo.toml` — add `tree-sitter-html`, `pulldown-cmark`.
- `crates/ride-core/src/lib.rs` — declare `git` and `preview` modules.
- `crates/ride-core/src/theme.rs` — 3 new git color fields across all themes; new `colorblind` theme.
- `crates/ride-core/src/highlight.rs` — `TreeSitterLang::Html` + extension detection.
- `crates/ride-core/src/highlight/treesitter_hl.rs` — HTML grammar + `html_highlight`.
- `crates/ride-core/src/command.rs` — `TogglePreview` command.
- `crates/ride-tui/src/app.rs` — preview state, git baseline state + helpers, preview-scroll handling.
- `crates/ride-tui/src/ui.rs` — dispatch editor-vs-preview.
- `crates/ride-tui/src/ui_editor.rs` — git marker column + tint; welcome-screen keybinding.
- `crates/ride-tui/src/ui_status.rs` — git change counts.
- `keybindings.json`, `README.md`, `ROADMAP.md` — docs/bindings.

---

## Task 1: Theme git color fields (all existing themes)

**Files:**
- Modify: `crates/ride-core/src/theme.rs`

- [ ] **Step 1: Write the failing test**

Add to the `#[cfg(test)] mod tests` block in `crates/ride-core/src/theme.rs`:

```rust
    #[test]
    fn test_resolve_git_override() {
        let json = r##"{ "base": "dark", "ui": { "git_added": { "fg": "#0072B2" } } }"##;
        let config: ThemeConfig = serde_json::from_str(json).unwrap();
        let theme = Theme::resolve(&config);
        assert_eq!(theme.ui.git_added.fg.as_deref(), Some("#0072B2"));
        // Non-overridden git field keeps its base value
        assert_eq!(theme.ui.git_removed.fg.as_deref(), Some("red"));
    }
```

- [ ] **Step 2: Run test to verify it fails (compile error)**

Run: `cargo test -p ride-core test_resolve_git_override`
Expected: FAIL — `no field 'git_added' on type 'UiColors'` (compile error).

- [ ] **Step 3: Add the fields and wire them through**

In `crates/ride-core/src/theme.rs`, add to the end of `struct UiColors` (after `completion_item`):

```rust
    // Git change markers (fg = gutter marker color, bg = optional line tint)
    pub git_added: ColorStyle,
    pub git_modified: ColorStyle,
    pub git_removed: ColorStyle,
```

Add to the end of `struct UiOverride` (after `completion_item: Option<ColorStyle>,`):

```rust
    pub git_added: Option<ColorStyle>,
    pub git_modified: Option<ColorStyle>,
    pub git_removed: Option<ColorStyle>,
```

In `Theme::resolve`, after `apply_ui_cs!(completion_item);`:

```rust
                    apply_ui_cs!(git_added);
                    apply_ui_cs!(git_modified);
                    apply_ui_cs!(git_removed);
```

Now add the fields to each existing theme's `UiColors { ... }`. In `dark_ui()` (after `completion_item`):

```rust
        git_added: ColorStyle::fg_bg("green", "#0e2a1a"),
        git_modified: ColorStyle::fg_bg("yellow", "#2a2410"),
        git_removed: ColorStyle::fg("red"),
```

In `light_theme()`'s `ui`:

```rust
            git_added: ColorStyle::fg_bg("#116329", "#e6ffec"),
            git_modified: ColorStyle::fg_bg("#9a6700", "#fff8c5"),
            git_removed: ColorStyle::fg("#cf222e"),
```

In `monokai_theme()`'s `ui`:

```rust
            git_added: ColorStyle::fg_bg("#a6e22e", "#1e2a16"),
            git_modified: ColorStyle::fg_bg("#e6db74", "#2e2a16"),
            git_removed: ColorStyle::fg("#f92672"),
```

In `solarized_dark_theme()`'s `ui`:

```rust
            git_added: ColorStyle::fg_bg("#859900", "#0a2b22"),
            git_modified: ColorStyle::fg_bg("#b58900", "#0e2b1a"),
            git_removed: ColorStyle::fg("#dc322f"),
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `cargo test -p ride-core test_resolve_git_override`
Expected: PASS.

- [ ] **Step 5: Run the full core test suite (nothing else broke)**

Run: `cargo test -p ride-core`
Expected: all pass.

- [ ] **Step 6: Commit**

```bash
git add crates/ride-core/src/theme.rs
git commit -m "Add git_added/git_modified/git_removed theme color fields"
```

---

## Task 2: Colorblind dark theme

**Files:**
- Modify: `crates/ride-core/src/theme.rs`

- [ ] **Step 1: Write the failing tests**

In the `tests` module, add:

```rust
    #[test]
    fn test_colorblind_theme_registered() {
        assert!(Theme::builtin("colorblind").is_some());
        let names = Theme::builtin_names();
        assert!(names.contains(&"colorblind"));
    }
```

And update the existing `test_builtin_names` length assertion from `4` to `5`:

```rust
        assert_eq!(names.len(), 5);
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p ride-core test_colorblind_theme_registered test_builtin_names`
Expected: FAIL — `colorblind` not registered / wrong length.

- [ ] **Step 3: Add the theme constructor and register it**

In `crates/ride-core/src/theme.rs`, add `"colorblind" => Some(colorblind_dark_theme()),` inside `Theme::builtin`'s match (before `_ => None`), and add `"colorblind"` to the `builtin_names` slice:

```rust
        &["dark", "light", "monokai", "solarized-dark", "colorblind"]
```

Then add this constructor next to the other theme functions:

```rust
pub fn colorblind_dark_theme() -> Theme {
    Theme {
        name: "colorblind".to_string(),
        syntax: SyntaxColors {
            keyword: ColorStyle::fg_bold("#E69F00"),
            type_name: ColorStyle::fg("#56B4E9"),
            string: ColorStyle::fg("#009E73"),
            comment: ColorStyle::fg_italic("#999999"),
            number: ColorStyle::fg("#CC79A7"),
            function: ColorStyle::fg("#0072B2"),
            operator: ColorStyle::fg("#D55E00"),
            punctuation: ColorStyle::fg("#f0f0f0"),
            variable: ColorStyle::fg("#e0e0e0"),
            heading: ColorStyle::fg_bold("#56B4E9"),
            link: ColorStyle::fg_underline("#56B4E9"),
            emphasis: ColorStyle::italic_only(),
            mermaid_keyword: ColorStyle::fg_bold("#E69F00"),
            mermaid_arrow: ColorStyle::fg("#56B4E9"),
            log_error: ColorStyle::fg_bold("#D55E00"),
            log_warn: ColorStyle::fg("#E69F00"),
            log_info: ColorStyle::fg("#009E73"),
            log_debug: ColorStyle::fg("#999999"),
            log_timestamp: ColorStyle::fg("#56B4E9"),
            normal: ColorStyle::new(),
        },
        ui: UiColors {
            border_focused: "#56B4E9".into(),
            border_unfocused: "#666666".into(),
            line_number: "#666666".into(),
            line_number_active: ColorStyle::fg_bold("#F0E442"),
            bracket_match: ColorStyle::fg_bg_bold("#F0E442", "#333333"),
            fold_indicator: ColorStyle::fg_italic("#999999"),
            tilde_empty: "#666666".into(),
            wrap_gutter: "#666666".into(),
            diagnostic_error: ColorStyle::fg_bold("#D55E00"),
            diagnostic_warning: ColorStyle::fg_bold("#E69F00"),
            diagnostic_info: ColorStyle::fg("#56B4E9"),
            diagnostic_hint: ColorStyle::fg("#999999"),
            welcome_title: ColorStyle::fg_bold("#56B4E9"),
            welcome_key: ColorStyle::fg_bold("#F0E442"),
            welcome_desc: ColorStyle::fg("#f0f0f0"),
            welcome_section: ColorStyle::fg_bold("#E69F00"),
            status_bar_bg: "#333333".into(),
            status_label: ColorStyle::fg_bg_bold("#000000", "#56B4E9"),
            status_file: ColorStyle::fg_bg("#f0f0f0", "#333333"),
            status_position: ColorStyle::fg_bg("#cccccc", "#333333"),
            status_message: ColorStyle::fg("#E69F00"),
            status_hover: ColorStyle::fg("#56B4E9"),
            tab_active: ColorStyle::fg_bg_bold("#f0f0f0", "#333333"),
            tab_inactive: ColorStyle::fg("#999999"),
            tab_bar_bg: "#1a1a1a".into(),
            explorer_title: ColorStyle::fg_bold("#f0f0f0"),
            explorer_dir: ColorStyle::fg_bold("#56B4E9"),
            explorer_file: ColorStyle::fg("#f0f0f0"),
            explorer_selected: ColorStyle::fg_bg_bold("#000000", "#56B4E9"),
            search_label: ColorStyle::fg_bg_bold("#000000", "#F0E442"),
            search_query: ColorStyle::fg("#f0f0f0"),
            search_match_count: ColorStyle::fg("#999999"),
            fuzzy_border: "#56B4E9".into(),
            fuzzy_title: ColorStyle::fg_bold("#f0f0f0"),
            fuzzy_prompt: ColorStyle::fg("#F0E442"),
            fuzzy_match_count: ColorStyle::fg("#999999"),
            fuzzy_selected: ColorStyle::fg_bg_bold("#000000", "#56B4E9"),
            fuzzy_item: ColorStyle::fg("#f0f0f0"),
            goto_border: "#56B4E9".into(),
            goto_title: ColorStyle::fg_bold("#f0f0f0"),
            goto_prompt: ColorStyle::fg("#F0E442"),
            completion_border: "#666666".into(),
            completion_bg: "#1a1a1a".into(),
            completion_selected: ColorStyle::fg_bg_bold("#000000", "#56B4E9"),
            completion_item: ColorStyle::fg("#f0f0f0"),
            git_added: ColorStyle::fg_bg("#0072B2", "#0a1f2e"),
            git_modified: ColorStyle::fg_bg("#E69F00", "#2a2410"),
            git_removed: ColorStyle::fg("#D55E00"),
        },
    }
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p ride-core`
Expected: all pass (including `test_colorblind_theme_registered`, `test_builtin_names`).

- [ ] **Step 5: Commit**

```bash
git add crates/ride-core/src/theme.rs
git commit -m "Add colorblind-safe dark theme (Okabe-Ito palette)"
```

---

## Task 3: Git line-diff core (`diff_lines`)

**Files:**
- Create: `crates/ride-core/src/git.rs`
- Modify: `crates/ride-core/src/lib.rs`

- [ ] **Step 1: Create the module with types and a stub, and declare it**

Create `crates/ride-core/src/git.rs`:

```rust
use std::collections::HashSet;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineStatus {
    Unchanged,
    Added,
    Modified,
}

/// Per-line change state of the current buffer versus its committed (HEAD) version.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GitLineDiff {
    /// `status[i]` is the change state of current line `i`.
    pub status: Vec<LineStatus>,
    /// Indices of current lines that have one or more deleted lines immediately above them.
    pub deleted_before: HashSet<usize>,
}

#[derive(Debug, Clone, Copy)]
enum Op {
    Eq,
    Del,
    Ins,
}

/// Line-based LCS diff of `head` (committed) vs `current` (buffer) text.
pub fn diff_lines(head: &str, current: &str) -> GitLineDiff {
    let head_lines: Vec<&str> = head.lines().collect();
    let cur_lines: Vec<&str> = current.lines().collect();
    let n = head_lines.len();
    let m = cur_lines.len();

    // lcs[i][j] = length of LCS of head_lines[i..] and cur_lines[j..]
    let mut lcs = vec![vec![0usize; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            lcs[i][j] = if head_lines[i] == cur_lines[j] {
                lcs[i + 1][j + 1] + 1
            } else {
                lcs[i + 1][j].max(lcs[i][j + 1])
            };
        }
    }

    // Forward walk producing an op stream.
    let mut ops: Vec<Op> = Vec::new();
    let mut i = 0;
    let mut j = 0;
    while i < n && j < m {
        if head_lines[i] == cur_lines[j] {
            ops.push(Op::Eq);
            i += 1;
            j += 1;
        } else if lcs[i + 1][j] >= lcs[i][j + 1] {
            ops.push(Op::Del);
            i += 1;
        } else {
            ops.push(Op::Ins);
            j += 1;
        }
    }
    while i < n {
        ops.push(Op::Del);
        i += 1;
    }
    while j < m {
        ops.push(Op::Ins);
        j += 1;
    }

    // Group consecutive change hunks; classify inserts; record net deletions.
    let mut status = vec![LineStatus::Unchanged; m];
    let mut deleted_before: HashSet<usize> = HashSet::new();
    let mut cur_pos = 0usize;
    let mut k = 0;
    while k < ops.len() {
        match ops[k] {
            Op::Eq => {
                cur_pos += 1;
                k += 1;
            }
            _ => {
                let hunk_start = cur_pos;
                let mut dels = 0usize;
                let mut ins: Vec<usize> = Vec::new();
                loop {
                    match ops.get(k) {
                        Some(Op::Del) => {
                            dels += 1;
                            k += 1;
                        }
                        Some(Op::Ins) => {
                            ins.push(cur_pos);
                            cur_pos += 1;
                            k += 1;
                        }
                        _ => break,
                    }
                }
                for (rank, &idx) in ins.iter().enumerate() {
                    status[idx] = if rank < dels {
                        LineStatus::Modified
                    } else {
                        LineStatus::Added
                    };
                }
                if dels > ins.len() && m > 0 {
                    let anchor = (hunk_start + ins.len()).min(m - 1);
                    deleted_before.insert(anchor);
                }
            }
        }
    }

    GitLineDiff {
        status,
        deleted_before,
    }
}

/// Returns true if `working_dir` is inside a git work tree.
pub fn is_repo(working_dir: &Path) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(working_dir)
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Returns the committed (HEAD) text of `file_path`, or `None` if not a repo,
/// no HEAD, or the file is untracked.
pub fn head_blob(working_dir: &Path, file_path: &Path) -> Option<String> {
    let root_out = Command::new("git")
        .arg("-C")
        .arg(working_dir)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()?;
    if !root_out.status.success() {
        return None;
    }
    let root = String::from_utf8(root_out.stdout).ok()?;
    let root_path = Path::new(root.trim());

    let abs = if file_path.is_absolute() {
        file_path.to_path_buf()
    } else {
        working_dir.join(file_path)
    };
    let rel = abs.strip_prefix(root_path).ok()?;
    let rel_str = rel.to_str()?;

    let show = Command::new("git")
        .arg("-C")
        .arg(root_path)
        .arg("show")
        .arg(format!("HEAD:{}", rel_str))
        .output()
        .ok()?;
    if !show.status.success() {
        return None;
    }
    String::from_utf8(show.stdout).ok()
}
```

Add to `crates/ride-core/src/lib.rs` (keep alphabetical: after `pub mod fuzzy;`... place wherever consistent):

```rust
pub mod git;
```

- [ ] **Step 2: Write failing tests**

Append to `crates/ride-core/src/git.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identical_is_all_unchanged() {
        let d = diff_lines("a\nb\nc\n", "a\nb\nc\n");
        assert_eq!(d.status, vec![LineStatus::Unchanged; 3]);
        assert!(d.deleted_before.is_empty());
    }

    #[test]
    fn test_added_line_in_middle() {
        let d = diff_lines("a\nc\n", "a\nb\nc\n");
        assert_eq!(d.status[0], LineStatus::Unchanged);
        assert_eq!(d.status[1], LineStatus::Added);
        assert_eq!(d.status[2], LineStatus::Unchanged);
    }

    #[test]
    fn test_modified_line() {
        let d = diff_lines("a\nb\nc\n", "a\nB\nc\n");
        assert_eq!(d.status[1], LineStatus::Modified);
    }

    #[test]
    fn test_removed_line_marks_following() {
        let d = diff_lines("a\nb\nc\n", "a\nc\n");
        assert_eq!(d.status, vec![LineStatus::Unchanged; 2]);
        assert!(d.deleted_before.contains(&1));
    }

    #[test]
    fn test_added_at_end() {
        let d = diff_lines("a\n", "a\nb\n");
        assert_eq!(d.status[1], LineStatus::Added);
    }

    #[test]
    fn test_removed_at_end_marks_last_line() {
        let d = diff_lines("a\nb\n", "a\n");
        assert!(d.deleted_before.contains(&0));
    }
}
```

- [ ] **Step 3: Run tests to verify they pass**

Run: `cargo test -p ride-core git::tests`
Expected: all 6 pass. (If a test fails, the bug is in `diff_lines`, not the test.)

- [ ] **Step 4: Commit**

```bash
git add crates/ride-core/src/git.rs crates/ride-core/src/lib.rs
git commit -m "Add git line-diff core (diff_lines) and HEAD blob access"
```

---

## Task 4: Git baseline access integration test (optional but included)

**Files:**
- Modify: `crates/ride-core/src/git.rs` (test module)

- [ ] **Step 1: Write a failing integration test using a temp git repo**

Append inside `mod tests` in `crates/ride-core/src/git.rs`:

```rust
    #[test]
    fn test_head_blob_returns_committed_content() {
        use std::fs;
        use std::process::Command;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path();
        let run = |args: &[&str]| {
            Command::new("git").arg("-C").arg(path).args(args).output().unwrap();
        };
        run(&["init"]);
        run(&["config", "user.email", "t@t"]);
        run(&["config", "user.name", "t"]);
        fs::write(path.join("f.txt"), "committed\n").unwrap();
        run(&["add", "f.txt"]);
        run(&["commit", "-m", "init"]);

        assert!(is_repo(path));
        let blob = head_blob(path, std::path::Path::new("f.txt"));
        assert_eq!(blob.as_deref(), Some("committed\n"));

        // Untracked file -> None, but still a repo
        fs::write(path.join("new.txt"), "x\n").unwrap();
        assert!(head_blob(path, std::path::Path::new("new.txt")).is_none());
    }
```

- [ ] **Step 2: Run the test**

Run: `cargo test -p ride-core test_head_blob_returns_committed_content`
Expected: PASS. (`tempfile` is already a dev-dependency. Skip/ignore this task only if the build environment has no `git`.)

- [ ] **Step 3: Commit**

```bash
git add crates/ride-core/src/git.rs
git commit -m "Add integration test for git head_blob"
```

---

## Task 5: App git state + helpers

**Files:**
- Modify: `crates/ride-tui/src/app.rs`

- [ ] **Step 1: Add state fields**

In `struct App` (`crates/ride-tui/src/app.rs`), add after `theme: Theme,`:

```rust
    pub git_baselines: Vec<Option<String>>,
    pub git_is_repo: bool,
```

In `App::new`, the `working_dir` local exists before the struct literal. Initialize the fields in the `Self { ... }` literal (after `theme,`):

```rust
            git_baselines: Vec::new(),
            git_is_repo: ride_core::git::is_repo(&working_dir),
```

Note: `working_dir` is moved into the struct on the same literal. Compute `git_is_repo` from the local `working_dir` *before* that field is moved — placing `git_is_repo` after `working_dir` in the literal is fine because the value is computed from `&working_dir` at that point. If the borrow checker complains, add `let git_is_repo = ride_core::git::is_repo(&working_dir);` above the struct literal and use `git_is_repo,` shorthand.

- [ ] **Step 2: Add the baseline-refresh and diff helper methods**

Add these methods to `impl App` (near `active_highlighter`):

```rust
    /// Refresh the committed (HEAD) baseline for the active tab.
    pub fn refresh_git_baseline(&mut self) {
        while self.git_baselines.len() < self.tabs.tabs.len() {
            self.git_baselines.push(None);
        }
        let path = match self.tabs.active_buffer().and_then(|b| b.file_path.clone()) {
            Some(p) => p,
            None => return,
        };
        let baseline = ride_core::git::head_blob(&self.working_dir, &path);
        if let Some(slot) = self.git_baselines.get_mut(self.tabs.active) {
            *slot = baseline;
        }
    }

    /// Compute the git line diff for the active buffer, if in a repo.
    pub fn active_git_diff(&self) -> Option<ride_core::git::GitLineDiff> {
        let buf = self.tabs.active_buffer()?;
        let current = buf.rope.to_string();
        match self.git_baselines.get(self.tabs.active).and_then(|b| b.as_ref()) {
            Some(base) => Some(ride_core::git::diff_lines(base, &current)),
            None => {
                if self.git_is_repo {
                    // Untracked file inside a repo: treat every line as added.
                    let line_count = current.lines().count();
                    Some(ride_core::git::GitLineDiff {
                        status: vec![ride_core::git::LineStatus::Added; line_count],
                        deleted_before: std::collections::HashSet::new(),
                    })
                } else {
                    None
                }
            }
        }
    }
```

- [ ] **Step 3: Populate the baseline on open and on save**

In `App::open_file`, just before `self.focus = FocusPane::Editor;`, add:

```rust
                self.refresh_git_baseline();
```

In `App::handle_command`, find the `Command::Save` arm (it calls `buf.save()`). Immediately after a successful save, add:

```rust
                self.refresh_git_baseline();
```

(Place it after the save succeeds, alongside the existing status-message update for Save.)

- [ ] **Step 4: Build and run the core+tui suites**

Run: `cargo build && cargo test`
Expected: builds clean, all existing tests pass.

- [ ] **Step 5: Commit**

```bash
git add crates/ride-tui/src/app.rs
git commit -m "Wire git baseline state and diff helper into App"
```

---

## Task 6: Git marker column + line tint in the editor gutter

**Files:**
- Modify: `crates/ride-tui/src/ui_editor.rs`

This task has no unit test (it is ratatui rendering); it is gated by `cargo build`, `cargo test`, and manual verification.

- [ ] **Step 1: Import LineStatus**

At the top of `crates/ride-tui/src/ui_editor.rs`, add:

```rust
use ride_core::git::LineStatus;
```

- [ ] **Step 2: Widen the gutter and compute the git diff once**

Find (around line 91-93):

```rust
    let diag_gutter_width = 2u16; // "● " or "  "
    let line_num_width = 4u16;
    let gutter_width = diag_gutter_width + line_num_width;
```

Replace with:

```rust
    let git_gutter_width = 1u16; // "│" / "_" / " "
    let diag_gutter_width = 2u16; // "● " or "  "
    let line_num_width = 4u16;
    let gutter_width = git_gutter_width + diag_gutter_width + line_num_width;
```

Immediately after the `app.reparse_tree_sitter();` call (around line 88, **before** the `buf` immutable borrow at line ~108), compute and stash the diff and styles:

```rust
    let git_diff = app.active_git_diff();
    let git_added_style = to_style(&theme.ui.git_added);
    let git_modified_style = to_style(&theme.ui.git_modified);
    let git_removed_style = to_style(&theme.ui.git_removed);
    let git_added_tint = theme.ui.git_added.bg.as_ref().map(|c| parse_color(c));
    let git_modified_tint = theme.ui.git_modified.bg.as_ref().map(|c| parse_color(c));
```

- [ ] **Step 3: Compute per-line git marker + tint inside the render loop**

Inside the `while visual_row < viewport_h && buf_row < buf.line_count()` loop, after `let display_text = ...;` (around line 152), add:

```rust
        // Git change marker + tint for this row
        let (git_symbol, git_symbol_style, git_tint): (&str, Style, Option<ratatui::style::Color>) =
            match &git_diff {
                Some(d) => {
                    let st = d.status.get(buf_row).copied().unwrap_or(LineStatus::Unchanged);
                    match st {
                        LineStatus::Added => ("│", git_added_style, git_added_tint),
                        LineStatus::Modified => ("│", git_modified_style, git_modified_tint),
                        LineStatus::Unchanged => {
                            if d.deleted_before.contains(&buf_row) {
                                ("_", git_removed_style, None)
                            } else {
                                (" ", Style::default(), None)
                            }
                        }
                    }
                }
                None => (" ", Style::default(), None),
            };
```

- [ ] **Step 4: Apply the tint to the line style map**

After the diagnostic-underline loop that ends around line 244 (after the `for &(dstart, dend) in &diag_ranges { ... }` block), add:

```rust
        // Git line tint: subtle background across the whole line
        if let Some(bg) = git_tint {
            for slot in style_map.iter_mut() {
                *slot = slot.bg(bg);
            }
        }
```

- [ ] **Step 5: Prepend the git marker in all three gutter render sites**

In the **folded-line** branch (around line 258, where `spans` starts with `Span::styled(diag_symbol, ...)`), change the initial vec to lead with the git marker:

```rust
                    let mut spans = vec![
                        Span::styled(git_symbol, git_symbol_style),
                        Span::styled(diag_symbol, diag_symbol_style),
                        Span::styled(line_num, line_num_style),
                    ];
```

In the **first-chunk** branch (around line 323-344), the code pushes `diag_symbol` then the fold-indicator line number. Before the `spans.push(Span::styled(diag_symbol, diag_symbol_style));` line, insert:

```rust
                spans.push(Span::styled(git_symbol, git_symbol_style));
```

The continuation-chunk blank gutter (line ~346) and the tilde fill (line ~393) both use `width = gutter_width`, which now includes the git column, so they need no further change.

- [ ] **Step 6: Build, test, and manually verify**

Run: `cargo build && cargo test`
Expected: builds clean, all tests pass.

Manual verification:
```bash
cargo run -- crates/ride-core/src/git.rs
```
In the running editor: edit a tracked line (a `│` marker + subtle tint appears on it), add a new line (marker on the new line), delete a line (a `_` marker appears on the line below the deletion). Open a file outside any git repo and confirm the gutter shows no markers/tint. Press Ctrl+Q to quit.

- [ ] **Step 7: Commit**

```bash
git add crates/ride-tui/src/ui_editor.rs
git commit -m "Show git change markers and line tint in the editor gutter"
```

---

## Task 7: Git change counts in the status bar

**Files:**
- Modify: `crates/ride-tui/src/ui_status.rs`

- [ ] **Step 1: Add the counts segment**

In `render_status` (`crates/ride-tui/src/ui_status.rs`), after the `Ln/Col` position span block (after line 24, before the diagnostics block), add:

```rust
    // Git change counts (+added ~modified -removed)
    if let Some(diff) = app.active_git_diff() {
        let added = diff
            .status
            .iter()
            .filter(|s| **s == ride_core::git::LineStatus::Added)
            .count();
        let modified = diff
            .status
            .iter()
            .filter(|s| **s == ride_core::git::LineStatus::Modified)
            .count();
        let removed = diff.deleted_before.len();
        if added + modified + removed > 0 {
            spans.push(Span::styled(
                format!(" +{} ~{} -{} ", added, modified, removed),
                to_style(&theme.ui.git_added),
            ));
        }
    }
```

- [ ] **Step 2: Build and test**

Run: `cargo build && cargo test`
Expected: builds clean, tests pass.

- [ ] **Step 3: Manually verify**

Run `cargo run -- crates/ride-core/src/git.rs`, make an edit, and confirm the status bar shows `+a ~m -d` counts updating. Quit with Ctrl+Q.

- [ ] **Step 4: Commit**

```bash
git add crates/ride-tui/src/ui_status.rs
git commit -m "Show git +added ~modified -removed counts in the status bar"
```

---

## Task 8: HTML syntax highlighting

**Files:**
- Modify: `crates/ride-core/Cargo.toml`
- Modify: `crates/ride-core/src/highlight.rs`
- Modify: `crates/ride-core/src/highlight/treesitter_hl.rs`

- [ ] **Step 1: Add the grammar dependency**

In `crates/ride-core/Cargo.toml`, under `[dependencies]`, after `tree-sitter-cpp = "0.23"`:

```toml
tree-sitter-html = "0.23"
```

(Use the `0.23.x` line — it builds against `tree-sitter = "0.24"`, matching the other `0.23` grammars. If `cargo build` reports an ABI/version mismatch, pick the `tree-sitter-html` release whose `tree-sitter` dependency is `0.24`.)

- [ ] **Step 2: Write failing detection tests**

In `crates/ride-core/src/highlight.rs`, add (create a `#[cfg(test)] mod tests` block if none exists; otherwise append):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_html_extension_detected() {
        assert_eq!(
            detect_highlighter(Path::new("index.html")),
            HighlighterType::TreeSitter(TreeSitterLang::Html)
        );
        assert_eq!(
            detect_highlighter(Path::new("page.htm")),
            HighlighterType::TreeSitter(TreeSitterLang::Html)
        );
    }
}
```

- [ ] **Step 3: Run to verify failure**

Run: `cargo test -p ride-core test_html_extension_detected`
Expected: FAIL — `TreeSitterLang` has no variant `Html`.

- [ ] **Step 4: Add the variant and detection**

In `crates/ride-core/src/highlight.rs`, add `Html,` to `enum TreeSitterLang` (after `Cpp,`). In `detect_highlighter`, add before the `Some("md") => ...` arm:

```rust
        Some("html" | "htm") => HighlighterType::TreeSitter(TreeSitterLang::Html),
```

- [ ] **Step 5: Wire the grammar and highlight function**

In `crates/ride-core/src/highlight/treesitter_hl.rs`:

`get_language` — add:
```rust
            TreeSitterLang::Html => Some(tree_sitter_html::LANGUAGE.into()),
```

`lang_name` — add:
```rust
            TreeSitterLang::Html => "html",
```

`scope_aware_highlight` match — add:
```rust
            TreeSitterLang::Html => self.html_highlight(node_kind, parent_kind),
```

Add the method (next to `markdown_highlight`):
```rust
    fn html_highlight(&self, node_kind: &str, _parent_kind: Option<&str>) -> HighlightKind {
        match node_kind {
            "comment" => HighlightKind::Comment,
            "tag_name" | "erroneous_end_tag_name" => HighlightKind::Type,
            "attribute_name" => HighlightKind::Variable,
            "attribute_value" | "quoted_attribute_value" | "\"" | "'" => HighlightKind::String,
            "doctype" => HighlightKind::Keyword,
            "<" | ">" | "</" | "/>" | "=" => HighlightKind::Punctuation,
            _ => HighlightKind::Normal,
        }
    }
```

- [ ] **Step 6: Write a highlight-output test**

Append to the `tests` module in `crates/ride-core/src/highlight.rs`:

```rust
    #[test]
    fn test_html_highlights_tag_name() {
        use crate::highlight::treesitter_hl::TreeSitterHighlighter;
        let mut hl = TreeSitterHighlighter::new(TreeSitterLang::Html).unwrap();
        let src = "<p class=\"x\">hi</p>";
        hl.parse(src);
        let spans = hl.highlight_line(src, 0);
        // At least one Type span (the tag name) and one String span (the attr value).
        assert!(spans.iter().any(|s| s.kind == HighlightKind::Type));
        assert!(spans.iter().any(|s| s.kind == HighlightKind::String));
    }
```

(`treesitter_hl` is a public module — confirm `highlight_line` and `parse` are reachable; they are `pub`.)

- [ ] **Step 7: Run tests to verify they pass**

Run: `cargo test -p ride-core`
Expected: all pass (detection + HTML highlight output).

- [ ] **Step 8: Commit**

```bash
git add crates/ride-core/Cargo.toml crates/ride-core/src/highlight.rs crates/ride-core/src/highlight/treesitter_hl.rs Cargo.lock
git commit -m "Add HTML tree-sitter syntax highlighting"
```

---

## Task 9: Markdown render model (`render_markdown`)

**Files:**
- Modify: `crates/ride-core/Cargo.toml`
- Modify: `crates/ride-core/src/lib.rs`
- Create: `crates/ride-core/src/preview.rs`

- [ ] **Step 1: Add the dependency and declare the module**

In `crates/ride-core/Cargo.toml`, under `[dependencies]`:

```toml
pulldown-cmark = "0.12"
```

In `crates/ride-core/src/lib.rs`, add:

```rust
pub mod preview;
```

- [ ] **Step 2: Create the module**

Create `crates/ride-core/src/preview.rs`:

```rust
use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreviewStyle {
    Normal,
    Heading(u8),
    Bold,
    Italic,
    Code,
    Link,
    ListItem,
    BlockQuote,
    Rule,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewSpan {
    pub text: String,
    pub style: PreviewStyle,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PreviewLine {
    pub spans: Vec<PreviewSpan>,
}

fn heading_u8(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn inline_style(heading: Option<u8>, bold: usize, italic: usize, in_link: bool) -> PreviewStyle {
    if let Some(l) = heading {
        PreviewStyle::Heading(l)
    } else if in_link {
        PreviewStyle::Link
    } else if bold > 0 {
        PreviewStyle::Bold
    } else if italic > 0 {
        PreviewStyle::Italic
    } else {
        PreviewStyle::Normal
    }
}

/// Render Markdown source into a UI-agnostic model of styled terminal lines.
pub fn render_markdown(source: &str) -> Vec<PreviewLine> {
    let mut lines: Vec<PreviewLine> = Vec::new();
    let mut cur: Vec<PreviewSpan> = Vec::new();

    let mut bold = 0usize;
    let mut italic = 0usize;
    let mut in_link = false;
    let mut heading: Option<u8> = None;
    let mut in_code_block = false;
    let mut list_stack: Vec<Option<u64>> = Vec::new();

    let mut flush = |cur: &mut Vec<PreviewSpan>, lines: &mut Vec<PreviewLine>| {
        if !cur.is_empty() {
            lines.push(PreviewLine { spans: std::mem::take(cur) });
        }
    };

    for ev in Parser::new(source) {
        match ev {
            Event::Start(Tag::Heading { level, .. }) => {
                let l = heading_u8(level);
                heading = Some(l);
                cur.push(PreviewSpan {
                    text: format!("{} ", "#".repeat(l as usize)),
                    style: PreviewStyle::Heading(l),
                });
            }
            Event::Start(Tag::Emphasis) => italic += 1,
            Event::Start(Tag::Strong) => bold += 1,
            Event::Start(Tag::BlockQuote(_)) => {
                cur.push(PreviewSpan { text: "▌ ".to_string(), style: PreviewStyle::BlockQuote });
            }
            Event::Start(Tag::CodeBlock(_)) => in_code_block = true,
            Event::Start(Tag::List(start)) => list_stack.push(start),
            Event::Start(Tag::Item) => {
                let indent = "  ".repeat(list_stack.len().saturating_sub(1));
                let marker = match list_stack.last().copied().flatten() {
                    Some(n) => format!("{}. ", n),
                    None => "• ".to_string(),
                };
                cur.push(PreviewSpan {
                    text: format!("{}{}", indent, marker),
                    style: PreviewStyle::ListItem,
                });
            }
            Event::Start(Tag::Link { .. }) => in_link = true,

            Event::End(TagEnd::Heading(_)) => {
                heading = None;
                flush(&mut cur, &mut lines);
            }
            Event::End(TagEnd::Paragraph) => flush(&mut cur, &mut lines),
            Event::End(TagEnd::Emphasis) => italic = italic.saturating_sub(1),
            Event::End(TagEnd::Strong) => bold = bold.saturating_sub(1),
            Event::End(TagEnd::BlockQuote(_)) => flush(&mut cur, &mut lines),
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
                flush(&mut cur, &mut lines);
            }
            Event::End(TagEnd::List(_)) => {
                list_stack.pop();
            }
            Event::End(TagEnd::Item) => flush(&mut cur, &mut lines),
            Event::End(TagEnd::Link) => in_link = false,

            Event::Text(t) => {
                if in_code_block {
                    let parts: Vec<&str> = t.split('\n').collect();
                    for (k, part) in parts.iter().enumerate() {
                        if k > 0 {
                            flush(&mut cur, &mut lines);
                        }
                        if !part.is_empty() {
                            cur.push(PreviewSpan {
                                text: part.to_string(),
                                style: PreviewStyle::Code,
                            });
                        }
                    }
                } else {
                    let style = inline_style(heading, bold, italic, in_link);
                    cur.push(PreviewSpan { text: t.to_string(), style });
                }
            }
            Event::Code(t) => {
                cur.push(PreviewSpan { text: t.to_string(), style: PreviewStyle::Code });
            }
            Event::SoftBreak | Event::HardBreak => {
                cur.push(PreviewSpan { text: " ".to_string(), style: PreviewStyle::Normal });
            }
            Event::Rule => {
                flush(&mut cur, &mut lines);
                cur.push(PreviewSpan { text: "─".repeat(40), style: PreviewStyle::Rule });
                flush(&mut cur, &mut lines);
            }
            _ => {}
        }
    }
    flush(&mut cur, &mut lines);
    lines
}
```

- [ ] **Step 3: Write failing tests**

Append to `crates/ride-core/src/preview.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn text_of(lines: &[PreviewLine]) -> String {
        lines
            .iter()
            .map(|l| l.spans.iter().map(|s| s.text.as_str()).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn test_heading_styled() {
        let lines = render_markdown("# Title");
        assert!(lines[0].spans.iter().any(|s| s.style == PreviewStyle::Heading(1)));
        assert!(text_of(&lines).contains("Title"));
    }

    #[test]
    fn test_bold_and_italic() {
        let lines = render_markdown("**b** and *i*");
        assert!(lines.iter().flat_map(|l| &l.spans).any(|s| s.style == PreviewStyle::Bold));
        assert!(lines.iter().flat_map(|l| &l.spans).any(|s| s.style == PreviewStyle::Italic));
    }

    #[test]
    fn test_unordered_list_marker() {
        let lines = render_markdown("- one\n- two");
        assert!(text_of(&lines).contains("• one"));
        assert!(text_of(&lines).contains("• two"));
    }

    #[test]
    fn test_ordered_list_marker() {
        let lines = render_markdown("1. first");
        assert!(text_of(&lines).contains("1. first"));
    }

    #[test]
    fn test_code_block() {
        let lines = render_markdown("```\nlet x = 1;\n```");
        assert!(lines.iter().flat_map(|l| &l.spans).any(|s| s.style == PreviewStyle::Code));
        assert!(text_of(&lines).contains("let x = 1;"));
    }

    #[test]
    fn test_blockquote() {
        let lines = render_markdown("> quoted");
        assert!(lines.iter().flat_map(|l| &l.spans).any(|s| s.style == PreviewStyle::BlockQuote));
        assert!(text_of(&lines).contains("quoted"));
    }

    #[test]
    fn test_link_text_preserved() {
        let lines = render_markdown("[click](http://example.com)");
        assert!(lines.iter().flat_map(|l| &l.spans).any(|s| s.style == PreviewStyle::Link));
        assert!(text_of(&lines).contains("click"));
    }

    #[test]
    fn test_thematic_break() {
        let lines = render_markdown("a\n\n---\n\nb");
        assert!(lines.iter().flat_map(|l| &l.spans).any(|s| s.style == PreviewStyle::Rule));
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p ride-core preview::tests`
Expected: all 8 pass. If the build fails on a `pulldown-cmark` API mismatch (the `Tag`/`TagEnd` enums shifted between minor versions), align the match arms with the pinned `0.12` API as the compiler indicates — the event model (Start/End/Text/Code/SoftBreak/Rule) is stable.

- [ ] **Step 5: Commit**

```bash
git add crates/ride-core/Cargo.toml crates/ride-core/src/lib.rs crates/ride-core/src/preview.rs Cargo.lock
git commit -m "Add Markdown render model (render_markdown) via pulldown-cmark"
```

---

## Task 10: Markdown preview UI, command, and wiring

**Files:**
- Create: `crates/ride-tui/src/ui_preview.rs`
- Modify: `crates/ride-core/src/command.rs`
- Modify: `crates/ride-tui/src/app.rs`
- Modify: `crates/ride-tui/src/ui.rs`
- Modify: `crates/ride-tui/src/main.rs` (module declaration)
- Modify: `keybindings.json`

- [ ] **Step 1: Add the `TogglePreview` command**

In `crates/ride-core/src/command.rs`:
- Add `TogglePreview,` to `enum Command` (e.g. after the `Folding` group).
- Add `TogglePreview,` to `enum SimpleCommand`.
- Add to `SimpleCommand::into_command`: `Self::TogglePreview => Command::TogglePreview,`.

- [ ] **Step 2: Add preview state to App**

In `crates/ride-tui/src/app.rs` `struct App`, add:

```rust
    pub preview_active: bool,
    pub preview_scroll: usize,
```

Initialize in `App::new`'s struct literal:

```rust
            preview_active: false,
            preview_scroll: 0,
```

- [ ] **Step 3: Handle the command and preview scrolling**

In `App::handle_command`, add a `Command::TogglePreview` arm:

```rust
            Command::TogglePreview => {
                let is_md = self.active_highlighter()
                    == HighlighterType::TreeSitter(ride_core::highlight::TreeSitterLang::Markdown);
                if is_md {
                    self.preview_active = !self.preview_active;
                    self.preview_scroll = 0;
                } else {
                    self.status_message =
                        "Preview is only available for Markdown files".to_string();
                }
            }
```

In the same function, make the movement commands scroll the preview when it is active. At the very top of `handle_command`, before the main `match cmd`, add:

```rust
        if self.preview_active {
            match cmd {
                Command::MoveDown => {
                    self.preview_scroll = self.preview_scroll.saturating_add(1);
                    return;
                }
                Command::MoveUp => {
                    self.preview_scroll = self.preview_scroll.saturating_sub(1);
                    return;
                }
                Command::PageDown => {
                    self.preview_scroll =
                        self.preview_scroll.saturating_add(self.viewport_height.max(1));
                    return;
                }
                Command::PageUp => {
                    self.preview_scroll =
                        self.preview_scroll.saturating_sub(self.viewport_height.max(1));
                    return;
                }
                Command::TogglePreview => {
                    self.preview_active = false;
                    return;
                }
                _ => {}
            }
        }
```

- [ ] **Step 4: Create the preview renderer**

Create `crates/ride-tui/src/ui_preview.rs`:

```rust
use crate::app::App;
use crate::theme_style::to_style;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;
use ride_core::preview::{render_markdown, PreviewStyle};

pub fn render_preview(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let source = match app.tabs.active_buffer() {
        Some(buf) => buf.rope.to_string(),
        None => return,
    };

    let model = render_markdown(&source);

    let style_for = |ps: &PreviewStyle| -> Style {
        match ps {
            PreviewStyle::Heading(_) => to_style(&theme.syntax.heading),
            PreviewStyle::Link => to_style(&theme.syntax.link),
            PreviewStyle::Code => to_style(&theme.syntax.string),
            PreviewStyle::Bold => to_style(&theme.syntax.emphasis).add_modifier(Modifier::BOLD),
            PreviewStyle::Italic => to_style(&theme.syntax.emphasis).add_modifier(Modifier::ITALIC),
            PreviewStyle::BlockQuote => to_style(&theme.syntax.comment),
            PreviewStyle::ListItem => to_style(&theme.syntax.keyword),
            PreviewStyle::Rule => to_style(&theme.syntax.comment),
            PreviewStyle::Normal => to_style(&theme.syntax.normal),
        }
    };

    let lines: Vec<Line> = model
        .iter()
        .skip(app.preview_scroll)
        .map(|pl| {
            let spans: Vec<Span> = pl
                .spans
                .iter()
                .map(|s| Span::styled(s.text.clone(), style_for(&s.style)))
                .collect();
            Line::from(spans)
        })
        .collect();

    let paragraph = Paragraph::new(lines).wrap(ratatui::widgets::Wrap { trim: false });
    frame.render_widget(paragraph, area);
}
```

- [ ] **Step 5: Declare the module and dispatch to it**

In `crates/ride-tui/src/main.rs`, add the module declaration alongside the other `mod ui_*;` lines:

```rust
mod ui_preview;
```

In `crates/ride-tui/src/ui.rs`, add the import:

```rust
use crate::ui_preview::render_preview;
use ride_core::highlight::{HighlighterType, TreeSitterLang};
```

Add a content-dispatch helper at the bottom of `ui.rs`:

```rust
fn render_content(frame: &mut Frame, area: Rect, app: &mut App) {
    let is_md = app.active_highlighter()
        == HighlighterType::TreeSitter(TreeSitterLang::Markdown);
    if app.preview_active && is_md {
        render_preview(frame, area, app);
    } else {
        render_editor(frame, area, app);
    }
}
```

Add `use ratatui::layout::Rect;` to the imports if not already present (the file imports `Layout`; add `Rect`). Then replace the two `render_editor(frame, ..., app);` call sites (lines ~51 and ~53) with `render_content(frame, ..., app);`.

- [ ] **Step 6: Add the keybinding**

In `keybindings.json`, add to the `"editor"` array:

```json
    { "key": "ctrl+e", "command": "TogglePreview" },
```

- [ ] **Step 7: Build and test**

Run: `cargo build && cargo test`
Expected: builds clean, all tests pass.

- [ ] **Step 8: Manually verify**

Run `cargo run -- README.md`. Press `Ctrl+E`: the editor area switches to a rendered preview (heading styled, `•` bullets, links colored). Arrow keys scroll the preview; `Ctrl+E` returns to editing. Open a non-Markdown file, press `Ctrl+E`, and confirm the status bar shows "Preview is only available for Markdown files". Quit with Ctrl+Q.

- [ ] **Step 9: Commit**

```bash
git add crates/ride-tui/src/ui_preview.rs crates/ride-tui/src/main.rs crates/ride-tui/src/ui.rs crates/ride-tui/src/app.rs crates/ride-core/src/command.rs keybindings.json
git commit -m "Add Markdown preview toggle (Ctrl+E)"
```

---

## Task 11: Documentation

**Files:**
- Modify: `README.md`
- Modify: `ROADMAP.md`
- Modify: `crates/ride-tui/src/ui_editor.rs` (welcome screen)

- [ ] **Step 1: Update the README**

In `README.md`:
- Add to the keybindings table: `| Ctrl+E | Toggle Markdown preview |`.
- Add to the "Supported Languages" table: `| \`.html\`, \`.htm\` | HTML | tree-sitter |`.
- In the Themes section, change the built-in list to include `colorblind`: "Built-in: `dark` (default), `light`, `monokai`, `solarized-dark`, `colorblind` (red-green-safe)."
- Under the custom-theme override example, document the new git fields, e.g. add to the `ui` example: `"git_added": { "fg": "#0072B2" }` and mention `git_modified` / `git_removed`.
- Add a line to Highlights noting "git change markers in the gutter" and "Markdown preview".

- [ ] **Step 2: Update the welcome screen**

In `crates/ride-tui/src/ui_editor.rs`, in the welcome `vec!`, add under an appropriate section (e.g. after the LSP block or in Navigation):

```rust
            keybinding("Ctrl+E", "Toggle Markdown preview"),
```

- [ ] **Step 3: Update the ROADMAP**

In `ROADMAP.md`, remove or check off items now delivered if listed, and (optionally) note git integration and Markdown preview as done. (No code dependency; keep it brief.)

- [ ] **Step 4: Build and verify docs render**

Run: `cargo build`
Expected: builds clean (welcome-screen change compiles).

- [ ] **Step 5: Commit**

```bash
git add README.md ROADMAP.md crates/ride-tui/src/ui_editor.rs
git commit -m "Document HTML highlighting, Markdown preview, git markers, colorblind theme"
```

---

## Final verification

- [ ] **Run the full suite and a clippy pass**

Run: `cargo test && cargo clippy --all-targets -- -D warnings`
Expected: all tests pass; no clippy errors (the repo's CI enforces clippy — see `.github`).

- [ ] **Smoke test each feature in a real run**

```bash
cargo run -- .
```
- Open an `.html` file → tags/attributes are highlighted.
- Open a Markdown file, Ctrl+E → preview renders; Ctrl+E again → back to editing.
- Edit a tracked file → git markers + tint appear; status bar shows counts.
- Set `"theme": "colorblind"` in `settings.json`, reopen → palette applies, git added is blue not green.
```

---

## Notes for the implementer

- **Borrow ordering in `ui_editor.rs`:** `app.active_git_diff()` borrows `&self` and returns an **owned** `GitLineDiff`; call it before the long-lived `let buf = app.tabs.active_buffer().unwrap();` borrow (Task 6, Step 2). Do not call it while `buf` is borrowed.
- **`status` length vs `line_count`:** `diff_lines` uses `str::lines()`, which can differ from `buf.line_count()` by one on trailing-newline files. The renderer and status code use `.get(buf_row)` / counting, so a length mismatch never panics.
- **pulldown-cmark version churn:** the `Tag`/`TagEnd` split (vs the older single `Tag` for end events) and `BlockQuote(Option<_>)` / `Link { .. }` shapes are from the `0.12` line. If you pin a different version, adjust those match arms; the `Event` variants used here are stable.
- **tree-sitter-html ABI:** must match `tree-sitter = "0.24"`. If `cargo build` errors on language ABI, choose the `tree-sitter-html` patch release built against tree-sitter 0.24.
- **Commit hygiene:** per project convention, do **not** add a `Co-Authored-By` trailer to commit messages.
