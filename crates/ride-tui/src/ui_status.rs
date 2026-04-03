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
