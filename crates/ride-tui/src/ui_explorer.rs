use crate::app::{App, ExplorerInputMode};
use crate::theme_style::{parse_color, to_style};
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};
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

    // Explorer input prompt
    if let Some(mode) = &app.explorer_input_mode {
        let prompt = match mode {
            ExplorerInputMode::NewFile => "New file: ",
            ExplorerInputMode::NewFolder => "New folder: ",
            ExplorerInputMode::Rename => "Rename: ",
            ExplorerInputMode::ConfirmDelete => "Delete? (y/n): ",
        };
        let input_area = Rect::new(area.x, area.y + area.height.saturating_sub(1), area.width, 1);
        let text = format!("{}{}", prompt, app.explorer_input);
        let para = Paragraph::new(text).style(Style::default().fg(Color::Yellow).bg(Color::Black));
        frame.render_widget(Clear, input_area);
        frame.render_widget(para, input_area);
    }
}
