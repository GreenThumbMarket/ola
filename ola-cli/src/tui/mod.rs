/// TUI module for Ola - Terminal User Interface
/// Provides an interactive terminal interface with keyboard navigation

pub mod app;
pub mod events;
pub mod ui;

pub use app::App;
pub use events::{Event, EventHandler};

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;

/// Main TUI runner
pub async fn run() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and event handler
    let mut app = App::new()?;
    let event_handler = EventHandler::new(250);

    // Run the app
    let result = run_app(&mut terminal, &mut app, event_handler).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    event_handler: EventHandler,
) -> Result<()> {
    loop {
        // Draw UI
        terminal.draw(|f| ui::draw(f, app))?;

        // Handle events
        match event_handler.next()? {
            Event::Tick => app.on_tick(),
            Event::Key(key) => {
                if !app.handle_key(key).await? {
                    break;
                }
            }
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}