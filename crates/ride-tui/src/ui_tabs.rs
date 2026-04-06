use crate::app::App;
use crate::theme_style::{parse_color, to_style};
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Tabs;
use ratatui::Frame;

pub fn render_tabs(frame: &mut Frame, area: Rect, app: &App) {
    if app.tabs.tabs.is_empty() {
        return;
    }

    let theme = &app.theme;
    let active_style = to_style(&theme.ui.tab_active);
    let inactive_style = to_style(&theme.ui.tab_inactive);

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
                Line::from(Span::styled(title, active_style))
            } else {
                Line::from(Span::styled(title, inactive_style))
            }
        })
        .collect();

    let tabs = Tabs::new(titles)
        .style(Style::default().bg(parse_color(&theme.ui.tab_bar_bg)))
        .divider(Span::raw("│"));

    frame.render_widget(tabs, area);
}
