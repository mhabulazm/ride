use crate::app::App;
use crate::theme_style::{parse_color, to_style};
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

pub fn render_goto_line(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;

    let width = 30u16.min(area.width.saturating_sub(4));
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + 1;
    let popup = Rect::new(x, y, width, 3);

    frame.render_widget(Clear, popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(parse_color(&theme.ui.goto_border)))
        .title(" Go to Line ")
        .title_style(to_style(&theme.ui.goto_title));

    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let line = Line::from(vec![
        Span::styled(": ", to_style(&theme.ui.goto_prompt)),
        Span::raw(&app.goto_line_input),
    ]);

    frame.render_widget(Paragraph::new(line), inner);
}
