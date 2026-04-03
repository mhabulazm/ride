use crate::app::App;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn render_search(frame: &mut Frame, area: Rect, app: &App) {
    let mode = if app.search.across_files {
        "Search (files)"
    } else {
        "Search"
    };

    let match_info = if app.search.matches.is_empty() {
        String::new()
    } else {
        format!(
            " [{}/{}]",
            app.search.current + 1,
            app.search.matches.len()
        )
    };

    let line = Line::from(vec![
        Span::styled(
            format!(" {}: ", mode),
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            &app.search.query,
            Style::default().fg(Color::White),
        ),
        Span::styled(
            match_info,
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    let paragraph = Paragraph::new(line);
    frame.render_widget(paragraph, area);
}
