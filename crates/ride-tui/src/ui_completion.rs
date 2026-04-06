use crate::app::App;
use crate::theme_style::{parse_color, to_style};
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;
use ride_core::lsp::CompletionKind;

pub fn render_completion(frame: &mut Frame, editor_area: Rect, app: &App) {
    if !app.completion_active || app.completion_items.is_empty() {
        return;
    }

    let buf = match app.tabs.active_buffer() {
        Some(b) => b,
        None => return,
    };

    let theme = &app.theme;

    // Position the popup near the cursor
    let gutter_width = 6u16; // diag_gutter(2) + line_num(4)
    let cursor_x = editor_area.x + gutter_width + 1 + buf.cursor_col as u16;
    let cursor_y = editor_area.y + (buf.cursor_row.saturating_sub(buf.scroll_row)) as u16;

    let max_items = 10usize;
    let visible_count = app.completion_items.len().min(max_items);
    let popup_height = visible_count as u16 + 2; // +2 for borders
    let popup_width = app
        .completion_items
        .iter()
        .map(|i| i.label.len() + kind_icon(i.kind).len() + 2)
        .max()
        .unwrap_or(20)
        .min(50) as u16
        + 2; // +2 for borders

    // Place below cursor, or above if no room
    let popup_y = if cursor_y + 1 + popup_height <= editor_area.y + editor_area.height {
        cursor_y + 1
    } else {
        cursor_y.saturating_sub(popup_height)
    };
    let popup_x = cursor_x.min(editor_area.x + editor_area.width.saturating_sub(popup_width));

    let popup_area = Rect::new(
        popup_x,
        popup_y,
        popup_width.min(editor_area.width),
        popup_height.min(editor_area.height),
    );

    // Compute scroll offset for the completion list
    let scroll_offset = if app.completion_index >= max_items {
        app.completion_index - max_items + 1
    } else {
        0
    };

    let selected_style = to_style(&theme.ui.completion_selected);
    let item_style = to_style(&theme.ui.completion_item);

    let lines: Vec<Line> = app
        .completion_items
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(max_items)
        .map(|(i, item)| {
            let icon = kind_icon(item.kind);
            let style = if i == app.completion_index {
                selected_style
            } else {
                item_style
            };
            Line::from(vec![
                Span::styled(format!("{} ", icon), style),
                Span::styled(item.label.clone(), style),
            ])
        })
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(parse_color(&theme.ui.completion_border)))
        .style(Style::default().bg(parse_color(&theme.ui.completion_bg)));

    let paragraph = Paragraph::new(lines).block(block);

    frame.render_widget(Clear, popup_area);
    frame.render_widget(paragraph, popup_area);
}

fn kind_icon(kind: CompletionKind) -> &'static str {
    match kind {
        CompletionKind::Method => "m",
        CompletionKind::Function => "f",
        CompletionKind::Constructor => "c",
        CompletionKind::Field => ".",
        CompletionKind::Variable => "v",
        CompletionKind::Class => "C",
        CompletionKind::Interface => "I",
        CompletionKind::Module => "M",
        CompletionKind::Property => "p",
        CompletionKind::Keyword => "k",
        CompletionKind::Snippet => "s",
        CompletionKind::Text => "t",
        CompletionKind::Other => " ",
    }
}
