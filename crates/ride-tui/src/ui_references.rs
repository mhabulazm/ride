use crate::app::App;
use crate::theme_style::{parse_color, to_style};
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

pub fn render_references(frame: &mut Frame, content_area: Rect, app: &App) {
    if !app.reference_active || app.reference_locations.is_empty() {
        return;
    }

    let theme = &app.theme;

    // Centered popup
    let popup_width = (content_area.width * 3 / 4).max(40).min(content_area.width);
    let max_items = 15usize;
    let visible_count = app.reference_locations.len().min(max_items);
    let popup_height = (visible_count as u16 + 2).min(content_area.height);

    let popup_x = content_area.x + (content_area.width.saturating_sub(popup_width)) / 2;
    let popup_y = content_area.y + (content_area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    let scroll_offset = if app.reference_index >= max_items {
        app.reference_index - max_items + 1
    } else {
        0
    };

    let selected_style = to_style(&theme.ui.fuzzy_selected);
    let item_style = to_style(&theme.ui.fuzzy_item);

    let lines: Vec<Line> = app
        .reference_locations
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(max_items)
        .map(|(i, loc)| {
            let file_name = loc
                .file
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| loc.file.to_string_lossy().to_string());
            let text = format!(" {}:{}:{}", file_name, loc.line + 1, loc.col + 1);
            let style = if i == app.reference_index {
                selected_style
            } else {
                item_style
            };
            Line::from(vec![Span::styled(text, style)])
        })
        .collect();

    let title = format!(" References ({}) ", app.reference_locations.len());
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(parse_color(&theme.ui.fuzzy_border)))
        .title(title);

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(Clear, popup_area);
    frame.render_widget(paragraph, popup_area);
}
