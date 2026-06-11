use crate::app::App;
use crate::theme_style::to_style;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;
use ride_core::preview::{render_markdown, PreviewStyle};

pub fn render_preview(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let source = match app.tabs.active_buffer() {
        Some(buf) => buf.rope.to_string(),
        None => return,
    };

    let model = render_markdown(&source);

    let style_for = |ps: &PreviewStyle| -> Style {
        match ps {
            PreviewStyle::Heading(_) => to_style(&theme.syntax.heading),
            PreviewStyle::Link => to_style(&theme.syntax.link),
            PreviewStyle::Code => to_style(&theme.syntax.string),
            PreviewStyle::Bold => to_style(&theme.syntax.emphasis).add_modifier(Modifier::BOLD),
            PreviewStyle::Italic => to_style(&theme.syntax.emphasis).add_modifier(Modifier::ITALIC),
            PreviewStyle::BlockQuote => to_style(&theme.syntax.comment),
            PreviewStyle::ListItem => to_style(&theme.syntax.keyword),
            PreviewStyle::Rule => to_style(&theme.syntax.comment),
            PreviewStyle::Normal => to_style(&theme.syntax.normal),
        }
    };

    let lines: Vec<Line> = model
        .iter()
        .skip(app.preview_scroll)
        .map(|pl| {
            let spans: Vec<Span> = pl
                .spans
                .iter()
                .map(|s| Span::styled(s.text.clone(), style_for(&s.style)))
                .collect();
            Line::from(spans)
        })
        .collect();

    let paragraph = Paragraph::new(lines).wrap(ratatui::widgets::Wrap { trim: false });
    frame.render_widget(paragraph, area);
}
