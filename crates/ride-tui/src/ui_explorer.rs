use crate::app::App;
use crate::theme_style::{parse_color, to_style};
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;
use ride_core::command::FocusPane;

pub fn render_explorer(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let border_style = if app.focus == FocusPane::Explorer {
        Style::default().fg(parse_color(&theme.ui.border_focused))
    } else {
        Style::default().fg(parse_color(&theme.ui.border_unfocused))
    };

    let block = Block::default()
        .borders(Borders::RIGHT)
        .border_style(border_style)
        .title(" Files ")
        .title_style(to_style(&theme.ui.explorer_title));

    let inner = block.inner(area);

    let selected_style = to_style(&theme.ui.explorer_selected);
    let dir_style = to_style(&theme.ui.explorer_dir);
    let file_style = to_style(&theme.ui.explorer_file);

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
                selected_style
            } else if entry.is_dir {
                dir_style
            } else {
                file_style
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
