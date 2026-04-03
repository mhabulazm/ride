use crate::app::App;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn render_status(frame: &mut Frame, area: Rect, app: &App) {
    let mut spans = vec![Span::styled(
        " RIDE ",
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )];

    if let Some(buf) = app.tabs.active_buffer() {
        let file_name = buf.file_name();
        let dirty = if buf.dirty { " ●" } else { "" };
        spans.push(Span::styled(
            format!(" {}{} ", file_name, dirty),
            Style::default().fg(Color::White).bg(Color::DarkGray),
        ));
        spans.push(Span::styled(
            format!(" Ln {}, Col {} ", buf.cursor_row + 1, buf.cursor_col + 1),
            Style::default().fg(Color::Gray).bg(Color::DarkGray),
        ));
    }

    // Show diagnostic at cursor line
    if let Some(buf) = app.tabs.active_buffer() {
        if let Some(ref path) = buf.file_path {
            let diags = app.lsp.get_diagnostics_for_line(path, buf.cursor_row);
            if let Some(d) = diags.first() {
                let color = match d.severity {
                    ride_core::lsp::DiagnosticSeverity::Error => Color::Red,
                    ride_core::lsp::DiagnosticSeverity::Warning => Color::Yellow,
                    _ => Color::Cyan,
                };
                spans.push(Span::styled(
                    format!("  {} ", d.message),
                    Style::default().fg(color),
                ));
            }
        }
    }

    // Show hover info if available
    if let Some(ref hover) = app.hover_display {
        spans.push(Span::styled(
            format!("  {} ", hover.lines().next().unwrap_or("")),
            Style::default().fg(Color::Cyan),
        ));
    }

    if !app.status_message.is_empty() {
        spans.push(Span::styled(
            format!("  {} ", &app.status_message),
            Style::default().fg(Color::Yellow),
        ));
    }

    let status = Paragraph::new(Line::from(spans))
        .style(Style::default().bg(Color::DarkGray));

    frame.render_widget(status, area);
}
