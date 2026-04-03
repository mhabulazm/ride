use crate::app::App;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};
use ratatui::Frame;

const MAX_VISIBLE: usize = 12;

pub fn render_fuzzy(frame: &mut Frame, area: Rect, app: &App) {
    // Center a popup over the content area
    let popup = centered_rect(50, MAX_VISIBLE as u16 + 3, area);
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" Open File (Ctrl+P) ")
        .title_style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        );

    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(inner);

    // Input line
    let match_count = app.fuzzy.filtered.len();
    let input_line = Line::from(vec![
        Span::styled("> ", Style::default().fg(Color::Yellow)),
        Span::raw(&app.fuzzy.query),
        Span::styled(
            format!("  ({} files)", match_count),
            Style::default().fg(Color::DarkGray),
        ),
    ]);
    frame.render_widget(Paragraph::new(input_line), chunks[0]);

    // Results list
    let visible_count = MAX_VISIBLE.min(app.fuzzy.filtered.len());
    let scroll_offset = if app.fuzzy.selected >= visible_count {
        app.fuzzy.selected - visible_count + 1
    } else {
        0
    };

    let items: Vec<ListItem> = app
        .fuzzy
        .filtered
        .iter()
        .skip(scroll_offset)
        .take(MAX_VISIBLE)
        .enumerate()
        .map(|(i, path)| {
            let display = app.fuzzy.display_path(path);
            let actual_idx = scroll_offset + i;
            let style = if actual_idx == app.fuzzy.selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(Span::styled(format!("  {}", display), style))
        })
        .collect();

    frame.render_widget(List::new(items), chunks[1]);
}

fn centered_rect(width_percent: u16, height: u16, area: Rect) -> Rect {
    let popup_width = area.width * width_percent / 100;
    let x = area.x + (area.width.saturating_sub(popup_width)) / 2;
    let y = area.y + 1;
    let h = height.min(area.height.saturating_sub(2));
    Rect::new(x, y, popup_width, h)
}
