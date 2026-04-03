use crate::app::App;
use crate::ui_editor::render_editor;
use crate::ui_explorer::render_explorer;
use crate::ui_fuzzy::render_fuzzy;
use crate::ui_goto::render_goto_line;
use ride_core::command::FocusPane;
use crate::ui_search::render_search;
use crate::ui_status::render_status;
use crate::ui_tabs::render_tabs;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::Frame;

pub fn render(frame: &mut Frame, app: &mut App) {
    let size = frame.area();

    // Main vertical layout: [tabs | content | search? | status]
    let mut vertical_constraints = vec![
        Constraint::Length(1), // tab bar
        Constraint::Min(1),   // content area
    ];
    if app.search.active {
        vertical_constraints.push(Constraint::Length(1)); // search bar
    }
    vertical_constraints.push(Constraint::Length(1)); // status bar

    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vertical_constraints)
        .split(size);

    // Tab bar
    render_tabs(frame, vertical_chunks[0], app);

    // Content: explorer | editor
    let content_area = vertical_chunks[1];
    app.viewport_height = content_area.height as usize;

    if app.explorer.visible {
        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(25), // explorer width
                Constraint::Min(1),    // editor
            ])
            .split(content_area);

        render_explorer(frame, horizontal_chunks[0], app);
        render_editor(frame, horizontal_chunks[1], app);
    } else {
        render_editor(frame, content_area, app);
    }

    // Search bar (if active)
    let status_idx = if app.search.active {
        render_search(frame, vertical_chunks[2], app);
        3
    } else {
        2
    };

    // Status bar
    render_status(frame, vertical_chunks[status_idx], app);

    // Fuzzy finder overlay
    if app.fuzzy.active {
        render_fuzzy(frame, content_area, app);
    }

    // Go-to-line overlay
    if app.focus == FocusPane::GoToLine {
        render_goto_line(frame, content_area, app);
    }
}
