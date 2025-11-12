use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::io;

use super::{app::App, events::EventHandler, ui};

/// Initialize the terminal with alternate screen and raw mode
pub fn init() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

/// Restore the terminal to its original state
pub fn restore() -> Result<()> {
    disable_raw_mode()?;
    execute!(
        io::stdout(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    Ok(())
}

/// Run the TUI application
pub fn run() -> Result<()> {
    // Initialize terminal
    let mut terminal = init()?;

    // Create app state
    let mut app = App::new()?;

    // Create event handler
    let event_handler = EventHandler::new();

    // Main loop
    let result = run_app(&mut terminal, &mut app, &event_handler);

    // Restore terminal
    restore()?;

    result
}

/// Main application loop
fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    event_handler: &EventHandler,
) -> Result<()> {
    loop {
        // Draw UI
        terminal.draw(|f| ui::render(f, app))?;

        // Handle events
        event_handler.handle_events(app)?;

        // Check if we should quit
        if app.should_quit {
            break;
        }
    }

    Ok(())
}
