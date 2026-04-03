use crate::app::App;
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
        let welcome = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "  RIDE — Rust IDE",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("  Open a file or directory to get started."),
            Line::from(""),
            Line::from("  Ctrl+B  Toggle file explorer"),
            Line::from("  Ctrl+Q  Quit"),
        ]);
        frame.render_widget(welcome, inner);
        return;
    }

    let hl_type = app.active_highlighter();

    let buf = app.tabs.active_buffer_mut().unwrap();

    let viewport_h = inner.height as usize;
    let line_num_width = 4u16; // space for line numbers
    let text_width = inner.width.saturating_sub(line_num_width + 1) as usize;

    buf.update_scroll(viewport_h, text_width);

    // Bracket matching: find the two positions to highlight
    let bracket_positions = buf.find_matching_bracket().map(|match_pos| {
        ((buf.cursor_row, buf.cursor_col), match_pos)
    });

    let mut lines = Vec::new();
    for row in buf.scroll_row..buf.scroll_row + viewport_h {
        if row >= buf.line_count() {
            // Empty line with tilde
            lines.push(Line::from(vec![
                Span::styled(
                    format!("{:>width$} ", "~", width = line_num_width as usize),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
            continue;
        }

        let line_text = buf.get_line(row).unwrap_or_default();
        let display_text = line_text.trim_end_matches('\n');

        // Line number — color by diagnostic severity if present
        let line_num = format!("{:>width$} ", row + 1, width = line_num_width as usize);
        let diag_severity = buf.file_path.as_ref().and_then(|p| {
            let diags = app.lsp.get_diagnostics_for_line(p, row);
            diags.into_iter().map(|d| d.severity).min_by_key(|s| match s {
                DiagnosticSeverity::Error => 0,
                DiagnosticSeverity::Warning => 1,
                DiagnosticSeverity::Info => 2,
                DiagnosticSeverity::Hint => 3,
            })
        });
        let line_num_style = if let Some(sev) = diag_severity {
            match sev {
                DiagnosticSeverity::Error => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                DiagnosticSeverity::Warning => Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                _ => Style::default().fg(Color::Cyan),
            }
        } else if row == buf.cursor_row {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let mut spans = vec![Span::styled(line_num, line_num_style)];

        // Collect bracket highlight columns for this row
        let mut bracket_cols: Vec<usize> = Vec::new();
        if let Some(((r1, c1), (r2, c2))) = bracket_positions {
            if r1 == row { bracket_cols.push(c1); }
            if r2 == row { bracket_cols.push(c2); }
        }

        // Apply syntax highlighting
        let hl_spans = highlight::highlight_line(hl_type, display_text, row);

        // Build a style map per byte position for the visible portion
        let text_bytes = display_text.as_bytes();
        let end = display_text.len();
        let mut style_map: Vec<Style> = vec![Style::default(); end];

        // Apply syntax highlighting to style map
        for hl in &hl_spans {
            let s = hl.start.min(end);
            let e = hl.end.min(end);
            let style = highlight_style(hl.kind);
            for pos in s..e {
                style_map[pos] = style;
            }
        }

        // Override bracket positions with bracket style
        let bracket_style = Style::default()
            .fg(Color::Yellow)
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD);
        for &col in &bracket_cols {
            if col < end {
                style_map[col] = bracket_style;
            }
        }

        // Render from scroll_col, merging consecutive chars with same style
        if buf.scroll_col < end {
            let mut current_style = style_map[buf.scroll_col];
            let mut current_text = String::new();

            for pos in buf.scroll_col..end {
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

        lines.push(Line::from(spans));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);

    // Position cursor
    let cursor_x =
        inner.x + line_num_width + 1 + (buf.cursor_col - buf.scroll_col) as u16;
    let cursor_y = inner.y + (buf.cursor_row - buf.scroll_row) as u16;
    if cursor_x < inner.x + inner.width && cursor_y < inner.y + inner.height {
        frame.set_cursor_position((cursor_x, cursor_y));
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
