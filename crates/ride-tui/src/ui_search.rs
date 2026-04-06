use crate::app::App;
use crate::theme_style::to_style;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn render_search(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;

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
            to_style(&theme.ui.search_label),
        ),
        Span::styled(
            &app.search.query,
            to_style(&theme.ui.search_query),
        ),
        Span::styled(
            match_info,
            to_style(&theme.ui.search_match_count),
        ),
    ]);

    let paragraph = Paragraph::new(line);
    frame.render_widget(paragraph, area);
}
