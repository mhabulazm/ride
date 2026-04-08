use crate::app::App;
use crate::theme_style::{parse_color, to_style};
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn render_status(frame: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;

    let mut spans = vec![Span::styled(" RIDE ", to_style(&theme.ui.status_label))];

    if let Some(buf) = app.tabs.active_buffer() {
        let file_name = buf.file_name();
        let dirty = if buf.dirty { " ●" } else { "" };
        spans.push(Span::styled(
            format!(" {}{} ", file_name, dirty),
            to_style(&theme.ui.status_file),
        ));
        spans.push(Span::styled(
            format!(" Ln {}, Col {} ", buf.cursor_row + 1, buf.cursor_col + 1),
            to_style(&theme.ui.status_position),
        ));
    }

    // Show diagnostic at cursor line
    if let Some(buf) = app.tabs.active_buffer() {
        if let Some(ref path) = buf.file_path {
            let diags = app.lsp.get_diagnostics_for_line(path, buf.cursor_row);
            if let Some(d) = diags.first() {
                let style = match d.severity {
                    ride_core::lsp::DiagnosticSeverity::Error => {
                        to_style(&theme.ui.diagnostic_error)
                    }
                    ride_core::lsp::DiagnosticSeverity::Warning => {
                        to_style(&theme.ui.diagnostic_warning)
                    }
                    ride_core::lsp::DiagnosticSeverity::Info => to_style(&theme.ui.diagnostic_info),
                    ride_core::lsp::DiagnosticSeverity::Hint => to_style(&theme.ui.diagnostic_hint),
                };
                spans.push(Span::styled(format!("  {} ", d.message), style));
            }
        }
    }

    // Show hover info if available
    if let Some(ref hover) = app.hover_display {
        spans.push(Span::styled(
            format!("  {} ", hover.lines().next().unwrap_or("")),
            to_style(&theme.ui.status_hover),
        ));
    }

    if !app.status_message.is_empty() {
        spans.push(Span::styled(
            format!("  {} ", &app.status_message),
            to_style(&theme.ui.status_message),
        ));
    }

    let status = Paragraph::new(Line::from(spans))
        .style(Style::default().bg(parse_color(&theme.ui.status_bar_bg)));

    frame.render_widget(status, area);
}
