mod app;
mod ui;
mod ui_editor;
mod ui_explorer;
mod ui_search;
mod ui_status;
mod ui_tabs;

use app::App;
use crossterm::event::{self, Event};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::stdout;
use std::path::PathBuf;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse CLI args
    let path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    let path = std::fs::canonicalize(&path).unwrap_or(path);

    // Setup terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new(&path);

    // Main loop
    while app.running {
        terminal.draw(|frame| {
            ui::render(frame, &mut app);
        })?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                // Skip release events
                if key.kind == crossterm::event::KeyEventKind::Press {
                    app.handle_key(key);
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}
