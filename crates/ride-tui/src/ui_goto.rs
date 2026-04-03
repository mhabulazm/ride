use crate::app::App;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

pub fn render_goto_line(frame: &mut Frame, area: Rect, app: &App) {
    let width = 30u16.min(area.width.saturating_sub(4));
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + 1;
    let popup = Rect::new(x, y, width, 3);

    frame.render_widget(Clear, popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" Go to Line ")
        .title_style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        );

    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let line = Line::from(vec![
        Span::styled(": ", Style::default().fg(Color::Yellow)),
        Span::raw(&app.goto_line_input),
    ]);

    frame.render_widget(Paragraph::new(line), inner);
}
