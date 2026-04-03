use crate::app::App;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;
use ride_core::command::FocusPane;
use ride_core::highlight::{self, HighlightKind};

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

        // Line number
        let line_num = format!("{:>width$} ", row + 1, width = line_num_width as usize);
        let line_num_style = if row == buf.cursor_row {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let mut spans = vec![Span::styled(line_num, line_num_style)];

        // Apply syntax highlighting
        let hl_spans = highlight::highlight_line(hl_type, display_text, row);

        if hl_spans.is_empty() {
            // No highlighting — render plain
            let visible = if buf.scroll_col < display_text.len() {
                &display_text[buf.scroll_col..]
            } else {
                ""
            };
            spans.push(Span::raw(visible.to_string()));
        } else {
            // Render with highlighting
            let text_bytes = display_text.as_bytes();
            let mut pos = buf.scroll_col;
            let end = display_text.len();

            // Sort spans by start position
            let mut sorted_spans = hl_spans.clone();
            sorted_spans.sort_by_key(|s| s.start);

            for hl in &sorted_spans {
                if hl.end <= pos || hl.start >= end {
                    continue;
                }
                // Gap before this span
                if hl.start > pos {
                    let gap_start = pos.max(buf.scroll_col);
                    let gap_end = hl.start.min(end);
                    if gap_start < gap_end {
                        spans.push(Span::raw(
                            String::from_utf8_lossy(&text_bytes[gap_start..gap_end]).to_string(),
                        ));
                    }
                }
                // The highlighted span
                let s_start = hl.start.max(pos).max(buf.scroll_col);
                let s_end = hl.end.min(end);
                if s_start < s_end {
                    spans.push(Span::styled(
                        String::from_utf8_lossy(&text_bytes[s_start..s_end]).to_string(),
                        highlight_style(hl.kind),
                    ));
                }
                pos = hl.end;
            }
            // Remaining text after last span
            if pos < end {
                let remaining_start = pos.max(buf.scroll_col);
                if remaining_start < end {
                    spans.push(Span::raw(
                        String::from_utf8_lossy(&text_bytes[remaining_start..end]).to_string(),
                    ));
                }
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
