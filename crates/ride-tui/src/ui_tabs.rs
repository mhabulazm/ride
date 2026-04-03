use crate::app::App;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Tabs;
use ratatui::Frame;

pub fn render_tabs(frame: &mut Frame, area: Rect, app: &App) {
    if app.tabs.tabs.is_empty() {
        return;
    }

    let titles: Vec<Line> = app
        .tabs
        .tabs
        .iter()
        .enumerate()
        .map(|(i, buf)| {
            let name = buf.file_name();
            let dirty = if buf.dirty { " ●" } else { "" };
            let title = format!(" {}{} ", name, dirty);
            if i == app.tabs.active {
                Line::from(Span::styled(
                    title,
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                ))
            } else {
                Line::from(Span::styled(
                    title,
                    Style::default().fg(Color::Gray),
                ))
            }
        })
        .collect();

    let tabs = Tabs::new(titles)
        .style(Style::default().bg(Color::Black))
        .divider(Span::raw("│"));

    frame.render_widget(tabs, area);
}
