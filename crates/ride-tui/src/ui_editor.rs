use crate::app::App;
use crate::theme_style::{parse_color, to_style};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;
use ride_core::command::FocusPane;
use ride_core::git::LineStatus;
use ride_core::highlight::{self, HighlightKind};
use ride_core::lsp::DiagnosticSeverity;
use ride_core::theme::Theme;
use std::collections::HashMap;

pub fn render_editor(frame: &mut Frame, area: Rect, app: &mut App) {
    let theme = app.theme.clone();
    let border_style = if app.focus == FocusPane::Editor {
        Style::default().fg(parse_color(&theme.ui.border_focused))
    } else {
        Style::default().fg(parse_color(&theme.ui.border_unfocused))
    };

    let block = Block::default()
        .borders(Borders::NONE)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.tabs.active_buffer().is_none() {
        let title_style = to_style(&theme.ui.welcome_title);
        let key_style = to_style(&theme.ui.welcome_key);
        let desc_style = to_style(&theme.ui.welcome_desc);
        let section_style = to_style(&theme.ui.welcome_section);

        let keybinding = |key: &str, desc: &str| -> Line {
            Line::from(vec![
                Span::raw("    "),
                Span::styled(format!("{:<22}", key), key_style),
                Span::styled(desc.to_string(), desc_style),
            ])
        };

        let welcome = vec![
            Line::from(""),
            Line::from(Span::styled("  RIDE — Rust IDE", title_style)),
            Line::from(""),
            Line::from(Span::styled(
                "  Open a file or directory to get started.",
                desc_style,
            )),
            Line::from(""),
            Line::from(Span::styled("  File", section_style)),
            keybinding("Ctrl+S", "Save"),
            keybinding("Ctrl+Q", "Quit"),
            keybinding("Ctrl+W", "Close tab"),
            Line::from(""),
            Line::from(Span::styled("  Navigation", section_style)),
            keybinding("Ctrl+P", "Fuzzy file finder"),
            keybinding("Ctrl+G", "Go to line"),
            keybinding("Ctrl+B", "Toggle file explorer"),
            keybinding("Ctrl+F", "Search in file"),
            keybinding("Ctrl+Shift+F", "Search across files"),
            keybinding("Ctrl+Left/Right", "Word-wise movement"),
            keybinding("Ctrl+Home/End", "Go to file start/end"),
            keybinding("Ctrl+PageDown", "Next tab"),
            keybinding("Ctrl+PageUp", "Previous tab"),
            keybinding("Alt+Left/Right", "Previous/Next tab"),
            Line::from(""),
            Line::from(Span::styled("  Editing", section_style)),
            keybinding("Ctrl+Z", "Undo"),
            keybinding("Enter", "New line (auto-indent)"),
            keybinding("Tab", "Insert tab"),
            Line::from(""),
            Line::from(Span::styled("  Preview", section_style)),
            keybinding("Ctrl+E", "Toggle Markdown preview"),
            Line::from(""),
            Line::from(Span::styled("  LSP", section_style)),
            keybinding("Ctrl+H", "Hover info"),
            keybinding("Ctrl+D", "Go to definition"),
            keybinding("Ctrl+Space", "Autocomplete"),
        ];

        let paragraph = Paragraph::new(welcome);
        frame.render_widget(paragraph, inner);
        return;
    }

    let hl_type = app.active_highlighter();

    // Reparse tree-sitter if buffer is dirty
    app.reparse_tree_sitter();

    let git_diff = app.active_git_diff();
    let git_added_style = to_style(&theme.ui.git_added);
    let git_modified_style = to_style(&theme.ui.git_modified);
    let git_removed_style = to_style(&theme.ui.git_removed);
    let git_added_tint = theme.ui.git_added.bg.as_ref().map(|c| parse_color(c));
    let git_modified_tint = theme.ui.git_modified.bg.as_ref().map(|c| parse_color(c));

    let viewport_h = inner.height as usize;
    let git_gutter_width = 1u16; // "│" / "_" / " "
    let diag_gutter_width = 2u16; // "● " or "  "
    let line_num_width = 4u16;
    let gutter_width = git_gutter_width + diag_gutter_width + line_num_width;
    let text_width = inner.width.saturating_sub(gutter_width + 1) as usize;

    if text_width == 0 {
        return;
    }

    // Mutable borrow to update scroll (vertical only, ignore horizontal for wrapping)
    {
        let buf = app.tabs.active_buffer_mut().unwrap();
        buf.update_scroll(viewport_h, text_width);
        // Reset horizontal scroll — soft wrap handles it
        buf.scroll_col = 0;
    }

    let buf = app.tabs.active_buffer().unwrap();

    // Bracket matching
    let bracket_positions = buf
        .find_matching_bracket()
        .map(|match_pos| ((buf.cursor_row, buf.cursor_col), match_pos));

    // Precompute tree-sitter spans for visible lines
    let ts_spans: HashMap<usize, Vec<highlight::HighlightSpan>> = (buf.scroll_row
        ..(buf.scroll_row + viewport_h).min(buf.line_count()))
        .filter_map(|row| app.ts_highlight_line(row).map(|spans| (row, spans)))
        .collect();

    let mut visual_lines: Vec<Line> = Vec::new();
    let mut cursor_screen_x: Option<u16> = None;
    let mut cursor_screen_y: Option<u16> = None;
    let mut visual_row: usize = 0;

    // Get fold state for active tab
    let fold_state = app.fold_states.get(app.tabs.active);

    // Pre-compute theme styles
    let line_num_color = parse_color(&theme.ui.line_number);
    let line_num_active_style = to_style(&theme.ui.line_number_active);
    let bracket_style = to_style(&theme.ui.bracket_match);
    let fold_style = to_style(&theme.ui.fold_indicator);
    let tilde_color = parse_color(&theme.ui.tilde_empty);
    let wrap_color = parse_color(&theme.ui.wrap_gutter);
    let diag_error_style = to_style(&theme.ui.diagnostic_error);
    let diag_warning_style = to_style(&theme.ui.diagnostic_warning);
    let diag_info_style = to_style(&theme.ui.diagnostic_info);
    let diag_hint_style = to_style(&theme.ui.diagnostic_hint);

    let mut buf_row = buf.scroll_row;
    while visual_row < viewport_h && buf_row < buf.line_count() {
        // Skip hidden (folded) lines
        if let Some(fs) = fold_state {
            if fs.is_line_hidden(buf_row) {
                buf_row += 1;
                continue;
            }
        }

        let line_text = buf.get_line(buf_row).unwrap_or_default();
        let display_text = line_text.trim_end_matches('\n');

        // Git change marker + tint for this row
        let (git_symbol, git_symbol_style, git_tint): (&str, Style, Option<ratatui::style::Color>) =
            match &git_diff {
                Some(d) => {
                    let st = d
                        .status
                        .get(buf_row)
                        .copied()
                        .unwrap_or(LineStatus::Unchanged);
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

        // Diagnostic severity for line number coloring and gutter symbol
        let diag_severity = buf.file_path.as_ref().and_then(|p| {
            let diags = app.lsp.get_diagnostics_for_line(p, buf_row);
            diags
                .into_iter()
                .map(|d| d.severity)
                .min_by_key(|s| match s {
                    DiagnosticSeverity::Error => 0,
                    DiagnosticSeverity::Warning => 1,
                    DiagnosticSeverity::Info => 2,
                    DiagnosticSeverity::Hint => 3,
                })
        });
        let (diag_symbol, diag_symbol_style) = match diag_severity {
            Some(DiagnosticSeverity::Error) => ("● ", diag_error_style),
            Some(DiagnosticSeverity::Warning) => ("▲ ", diag_warning_style),
            Some(DiagnosticSeverity::Info) => ("ℹ ", diag_info_style),
            Some(DiagnosticSeverity::Hint) => ("ℹ ", diag_hint_style),
            None => ("  ", Style::default()),
        };
        let line_num_style = if diag_severity.is_some() {
            diag_symbol_style
        } else if buf_row == buf.cursor_row {
            line_num_active_style
        } else {
            Style::default().fg(line_num_color)
        };

        // Collect diagnostic ranges for underline styling
        let diag_ranges: Vec<(usize, usize)> = buf
            .file_path
            .as_ref()
            .map(|p| {
                app.lsp
                    .get_diagnostics_for_line(p, buf_row)
                    .into_iter()
                    .map(|d| {
                        let end = if d.end_col > d.col {
                            d.end_col
                        } else {
                            d.col + 1
                        };
                        (d.col, end)
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Bracket highlight columns for this row
        let mut bracket_cols: Vec<usize> = Vec::new();
        if let Some(((r1, c1), (r2, c2))) = bracket_positions {
            if r1 == buf_row {
                bracket_cols.push(c1);
            }
            if r2 == buf_row {
                bracket_cols.push(c2);
            }
        }

        // Syntax highlighting — build style map for full line
        let hl_spans = ts_spans
            .get(&buf_row)
            .cloned()
            .unwrap_or_else(|| highlight::highlight_line(hl_type, display_text, buf_row));

        let text_bytes = display_text.as_bytes();
        let end = display_text.len();
        let mut style_map: Vec<Style> = vec![Style::default(); end];

        for hl in &hl_spans {
            let s = hl.start.min(end);
            let e = hl.end.min(end);
            let style = highlight_style(hl.kind, &theme);
            for slot in style_map.iter_mut().take(e).skip(s) {
                *slot = style;
            }
        }

        for &col in &bracket_cols {
            if col < end {
                style_map[col] = bracket_style;
            }
        }

        // Apply diagnostic underline styling
        for &(dstart, dend) in &diag_ranges {
            let underline_style_mod = Modifier::UNDERLINED;
            for slot in style_map.iter_mut().take(dend.min(end)).skip(dstart) {
                *slot = slot.add_modifier(underline_style_mod);
            }
        }

        // Git line tint: subtle background across the whole line
        if let Some(bg) = git_tint {
            for slot in style_map.iter_mut() {
                *slot = slot.bg(bg);
            }
        }

        // Handle folded regions: show first line with fold summary, skip content
        if let Some(fs) = fold_state {
            if let Some(region) = fs.get_region_at_start(buf_row) {
                if fs.is_folded(buf_row) {
                    let hidden = region.end_line - region.start_line;
                    let fold_indicator = "▶";
                    let line_num = format!(
                        "{}{:>width$} ",
                        fold_indicator,
                        buf_row + 1,
                        width = (line_num_width as usize).saturating_sub(1)
                    );
                    let mut spans = vec![
                        Span::styled(git_symbol, git_symbol_style),
                        Span::styled(diag_symbol, diag_symbol_style),
                        Span::styled(line_num, line_num_style),
                    ];

                    // Show truncated first line + fold summary
                    let preview_len = text_width.saturating_sub(25).min(end);
                    if preview_len > 0 {
                        let mut current_style = style_map[0];
                        let mut current_text = String::new();
                        for pos in 0..preview_len {
                            let s = style_map[pos];
                            if s != current_style {
                                if !current_text.is_empty() {
                                    spans.push(Span::styled(current_text.clone(), current_style));
                                    current_text.clear();
                                }
                                current_style = s;
                            }
                            current_text.push(text_bytes[pos] as char);
                        }
                        if !current_text.is_empty() {
                            spans.push(Span::styled(current_text, current_style));
                        }
                    }

                    spans.push(Span::styled(format!(" ... {} lines ", hidden), fold_style));

                    // Cursor on the fold start line
                    if buf_row == buf.cursor_row {
                        let col = buf.cursor_col.min(end);
                        cursor_screen_x = Some(inner.x + gutter_width + 1 + col as u16);
                        cursor_screen_y = Some(inner.y + visual_row as u16);
                    }

                    visual_lines.push(Line::from(spans));
                    visual_row += 1;
                    buf_row = region.end_line + 1;
                    continue;
                }
            }
        }

        // Split line into wrapped chunks
        let chunks = if end == 0 {
            vec![(0usize, 0usize)] // empty line still takes one visual row
        } else {
            let mut c = Vec::new();
            let mut start = 0;
            while start < end {
                let chunk_end = (start + text_width).min(end);
                c.push((start, chunk_end));
                start = chunk_end;
            }
            c
        };

        for (chunk_idx, &(chunk_start, chunk_end)) in chunks.iter().enumerate() {
            if visual_row >= viewport_h {
                break;
            }

            let mut spans: Vec<Span> = Vec::new();

            // Line number on first chunk, blank gutter on continuations
            if chunk_idx == 0 {
                // Git change marker
                spans.push(Span::styled(git_symbol, git_symbol_style));
                // Diagnostic gutter symbol
                spans.push(Span::styled(diag_symbol, diag_symbol_style));
                // Fold indicator
                let fold_indicator = if let Some(fs) = fold_state {
                    if fs.is_folded(buf_row) {
                        "▶"
                    } else if fs.is_fold_start(buf_row) {
                        "▼"
                    } else {
                        " "
                    }
                } else {
                    " "
                };
                let line_num = format!(
                    "{}{:>width$} ",
                    fold_indicator,
                    buf_row + 1,
                    width = (line_num_width as usize).saturating_sub(1)
                );
                spans.push(Span::styled(line_num, line_num_style));
            } else {
                let blank_gutter = format!("{:>width$} ", " ", width = gutter_width as usize);
                spans.push(Span::styled(blank_gutter, Style::default().fg(wrap_color)));
            }

            // Render text for this chunk with styles
            if chunk_start < chunk_end {
                let mut current_style = style_map[chunk_start];
                let mut current_text = String::new();

                for pos in chunk_start..chunk_end {
                    let s = style_map[pos];
                    if s != current_style {
                        if !current_text.is_empty() {
                            spans.push(Span::styled(current_text.clone(), current_style));
                            current_text.clear();
                        }
                        current_style = s;
                    }
                    current_text.push(text_bytes[pos] as char);
                }
                if !current_text.is_empty() {
                    spans.push(Span::styled(current_text, current_style));
                }
            }

            // Track cursor position
            if buf_row == buf.cursor_row {
                let cursor_col = buf.cursor_col;
                if cursor_col >= chunk_start && cursor_col <= chunk_end {
                    // Cursor is on this visual line (or at the end of this chunk)
                    if cursor_col < chunk_end || (chunk_idx == chunks.len() - 1) {
                        cursor_screen_x =
                            Some(inner.x + gutter_width + 1 + (cursor_col - chunk_start) as u16);
                        cursor_screen_y = Some(inner.y + visual_row as u16);
                    }
                }
            }

            visual_lines.push(Line::from(spans));
            visual_row += 1;
        }

        buf_row += 1;
    }

    // Fill remaining visual rows with tildes
    while visual_row < viewport_h {
        visual_lines.push(Line::from(vec![Span::styled(
            format!("{:>width$} ", "~", width = gutter_width as usize),
            Style::default().fg(tilde_color),
        )]));
        visual_row += 1;
    }

    let paragraph = Paragraph::new(visual_lines);
    frame.render_widget(paragraph, inner);

    // Position cursor
    if let (Some(cx), Some(cy)) = (cursor_screen_x, cursor_screen_y) {
        if cx < inner.x + inner.width && cy < inner.y + inner.height {
            frame.set_cursor_position((cx, cy));
        }
    }
}

fn highlight_style(kind: HighlightKind, theme: &Theme) -> Style {
    let cs = match kind {
        HighlightKind::Keyword => &theme.syntax.keyword,
        HighlightKind::Type => &theme.syntax.type_name,
        HighlightKind::String => &theme.syntax.string,
        HighlightKind::Comment => &theme.syntax.comment,
        HighlightKind::Number => &theme.syntax.number,
        HighlightKind::Function => &theme.syntax.function,
        HighlightKind::Operator => &theme.syntax.operator,
        HighlightKind::Punctuation => &theme.syntax.punctuation,
        HighlightKind::Variable => &theme.syntax.variable,
        HighlightKind::Heading => &theme.syntax.heading,
        HighlightKind::Link => &theme.syntax.link,
        HighlightKind::Emphasis => &theme.syntax.emphasis,
        HighlightKind::MermaidKeyword => &theme.syntax.mermaid_keyword,
        HighlightKind::MermaidArrow => &theme.syntax.mermaid_arrow,
        HighlightKind::LogError => &theme.syntax.log_error,
        HighlightKind::LogWarn => &theme.syntax.log_warn,
        HighlightKind::LogInfo => &theme.syntax.log_info,
        HighlightKind::LogDebug => &theme.syntax.log_debug,
        HighlightKind::LogTimestamp => &theme.syntax.log_timestamp,
        HighlightKind::Normal => &theme.syntax.normal,
    };
    to_style(cs)
}
