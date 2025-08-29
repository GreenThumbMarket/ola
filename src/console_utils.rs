use std::thread;
use std::time::Duration;
use console::Term;
use anyhow::Result;

pub struct ConsoleUtils {
    term: Term,
}

impl ConsoleUtils {
    pub fn new() -> Self {
        Self {
            term: Term::stdout(),
        }
    }

    /// Write a line to the terminal
    pub fn write_line(&self, text: &str) -> Result<()> {
        self.term.write_line(text)?;
        Ok(())
    }

    /// Clear the current line
    pub fn clear_line(&self) -> Result<()> {
        self.term.clear_line()?;
        Ok(())
    }

    /// Write a line, wait for a duration, then clear it
    pub fn write_line_with_delay(&self, text: &str, delay_ms: u64) -> Result<()> {
        self.write_line(text)?;
        thread::sleep(Duration::from_millis(delay_ms));
        self.clear_line()?;
        Ok(())
    }

    /// Move cursor up by n lines
    pub fn move_cursor_up(&self, n: usize) -> Result<()> {
        self.term.move_cursor_up(n)?;
        Ok(())
    }

    /// Move cursor down by n lines
    pub fn move_cursor_down(&self, n: usize) -> Result<()> {
        self.term.move_cursor_down(n)?;
        Ok(())
    }

    /// Clear the entire screen
    pub fn clear_screen(&self) -> Result<()> {
        self.term.clear_screen()?;
        Ok(())
    }

    /// Hide the cursor
    pub fn hide_cursor(&self) -> Result<()> {
        self.term.hide_cursor()?;
        Ok(())
    }

    /// Show the cursor
    pub fn show_cursor(&self) -> Result<()> {
        self.term.show_cursor()?;
        Ok(())
    }

    /// Get terminal size (width, height)
    pub fn size(&self) -> (u16, u16) {
        self.term.size()
    }

    /// Write text without a newline
    pub fn write(&self, text: &str) -> Result<()> {
        self.term.write_str(text)?;
        Ok(())
    }
}

impl Default for ConsoleUtils {
    fn default() -> Self {
        Self::new()
    }
}

/// Example function demonstrating the console features
pub fn demo_console_features() -> Result<()> {
    let console = ConsoleUtils::new();
    
    console.write_line("Hello World!")?;
    thread::sleep(Duration::from_millis(2000));
    console.clear_line()?;
    
    Ok(())
}

/// Loading animation example
pub fn loading_animation(message: &str, duration_ms: u64) -> Result<()> {
    let console = ConsoleUtils::new();
    let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let frame_duration = 100;
    let total_frames = duration_ms / frame_duration;
    
    console.hide_cursor()?;
    
    for i in 0..total_frames {
        let frame = frames[(i as usize) % frames.len()];
        console.write(&format!("\r{} {}", frame, message))?;
        thread::sleep(Duration::from_millis(frame_duration));
    }
    
    console.clear_line()?;
    console.show_cursor()?;
    Ok(())
}