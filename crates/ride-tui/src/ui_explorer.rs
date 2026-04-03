use crate::app::App;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;
use ride_core::command::FocusPane;

pub fn render_explorer(frame: &mut Frame, area: Rect, app: &App) {
    let border_style = if app.focus == FocusPane::Explorer {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .borders(Borders::RIGHT)
        .border_style(border_style)
        .title(" Files ")
        .title_style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD));

    let inner = block.inner(area);

    let items: Vec<ListItem> = app
        .explorer
        .entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let indent = "  ".repeat(entry.depth);
            let icon = if entry.is_dir {
                if entry.expanded {
                    "▼ "
                } else {
                    "▶ "
                }
            } else {
                "  "
            };

            let style = if i == app.explorer.selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else if entry.is_dir {
                Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(Line::from(vec![
                Span::raw(indent),
                Span::styled(format!("{}{}", icon, entry.name), style),
            ]))
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(block, area);
    frame.render_widget(list, inner);
}
