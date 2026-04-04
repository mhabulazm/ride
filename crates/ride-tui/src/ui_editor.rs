use crate::app::App;
use std::collections::HashMap;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;
use ride_core::command::FocusPane;
use ride_core::highlight::{self, HighlightKind};
use ride_core::lsp::DiagnosticSeverity;

pub fn render_editor(frame: &mut Frame, area: Rect, app: &mut App) {
    let border_style = if app.focus == FocusPane::Editor {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .borders(Borders::NONE)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.tabs.active_buffer().is_none() {
        let title_style = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);
        let key_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
        let desc_style = Style::default().fg(Color::White);
        let section_style = Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD);

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
            Line::from(Span::styled("  Open a file or directory to get started.", desc_style)),
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

    let viewport_h = inner.height as usize;
    let diag_gutter_width = 2u16; // "● " or "  "
    let line_num_width = 4u16;
    let gutter_width = diag_gutter_width + line_num_width;
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
    let bracket_positions = buf.find_matching_bracket().map(|match_pos| {
        ((buf.cursor_row, buf.cursor_col), match_pos)
    });

    // Precompute tree-sitter spans for visible lines
    let ts_spans: HashMap<usize, Vec<highlight::HighlightSpan>> = (buf.scroll_row
        ..(buf.scroll_row + viewport_h).min(buf.line_count()))
        .filter_map(|row| {
            app.ts_highlight_line(row).map(|spans| (row, spans))
        })
        .collect();

    let mut visual_lines: Vec<Line> = Vec::new();
    let mut cursor_screen_x: Option<u16> = None;
    let mut cursor_screen_y: Option<u16> = None;
    let mut visual_row: usize = 0;

    // Get fold state for active tab
    let fold_state = app.fold_states.get(app.tabs.active);

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

        // Diagnostic severity for line number coloring and gutter symbol
        let diag_severity = buf.file_path.as_ref().and_then(|p| {
            let diags = app.lsp.get_diagnostics_for_line(p, buf_row);
            diags.into_iter().map(|d| d.severity).min_by_key(|s| match s {
                DiagnosticSeverity::Error => 0,
                DiagnosticSeverity::Warning => 1,
                DiagnosticSeverity::Info => 2,
                DiagnosticSeverity::Hint => 3,
            })
        });
        let (diag_symbol, diag_symbol_style) = match diag_severity {
            Some(DiagnosticSeverity::Error) => (
                "● ",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Some(DiagnosticSeverity::Warning) => (
                "▲ ",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
            Some(DiagnosticSeverity::Info) => (
                "ℹ ",
                Style::default().fg(Color::Cyan),
            ),
            Some(DiagnosticSeverity::Hint) => (
                "ℹ ",
                Style::default().fg(Color::DarkGray),
            ),
            None => ("  ", Style::default()),
        };
        let line_num_style = if diag_severity.is_some() {
            diag_symbol_style
        } else if buf_row == buf.cursor_row {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
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
                        let end = if d.end_col > d.col { d.end_col } else { d.col + 1 };
                        (d.col, end)
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Bracket highlight columns for this row
        let mut bracket_cols: Vec<usize> = Vec::new();
        if let Some(((r1, c1), (r2, c2))) = bracket_positions {
            if r1 == buf_row { bracket_cols.push(c1); }
            if r2 == buf_row { bracket_cols.push(c2); }
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
            let style = highlight_style(hl.kind);
            for pos in s..e {
                style_map[pos] = style;
            }
        }

        let bracket_style = Style::default()
            .fg(Color::Yellow)
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD);
        for &col in &bracket_cols {
            if col < end {
                style_map[col] = bracket_style;
            }
        }

        // Apply diagnostic underline styling
        for &(dstart, dend) in &diag_ranges {
            let underline_style_mod = Modifier::UNDERLINED;
            for pos in dstart..dend.min(end) {
                style_map[pos] = style_map[pos].add_modifier(underline_style_mod);
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

                    spans.push(Span::styled(
                        format!(" ... {} lines ", hidden),
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::ITALIC),
                    ));

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
                spans.push(Span::styled(blank_gutter, Style::default().fg(Color::DarkGray)));
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
                        cursor_screen_x = Some(
                            inner.x + gutter_width + 1 + (cursor_col - chunk_start) as u16,
                        );
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
            Style::default().fg(Color::DarkGray),
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

fn highlight_style(kind: HighlightKind) -> Style {
    match kind {
        HighlightKind::Keyword => Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
        HighlightKind::Type => Style::default().fg(Color::Cyan),
        HighlightKind::String => Style::default().fg(Color::Green),
        HighlightKind::Comment => Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
        HighlightKind::Number => Style::default().fg(Color::Yellow),
        HighlightKind::Function => Style::default().fg(Color::Blue),
        HighlightKind::Operator => Style::default().fg(Color::Red),
        HighlightKind::Punctuation => Style::default().fg(Color::White),
        HighlightKind::Variable => Style::default().fg(Color::White),
        HighlightKind::Heading => Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        HighlightKind::Link => Style::default().fg(Color::Blue).add_modifier(Modifier::UNDERLINED),
        HighlightKind::Emphasis => Style::default().add_modifier(Modifier::ITALIC),
        HighlightKind::MermaidKeyword => Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
        HighlightKind::MermaidArrow => Style::default().fg(Color::Cyan),
        HighlightKind::LogError => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        HighlightKind::LogWarn => Style::default().fg(Color::Yellow),
        HighlightKind::LogInfo => Style::default().fg(Color::Green),
        HighlightKind::LogDebug => Style::default().fg(Color::DarkGray),
        HighlightKind::LogTimestamp => Style::default().fg(Color::Blue),
        HighlightKind::Normal => Style::default(),
    }
}
