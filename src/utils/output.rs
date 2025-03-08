// Output formatting utilities
use std::io::{self, Write};

/// Enum for defining ANSI color codes
pub enum Color {
    Red,
    Green,
    Yellow,
    Blue,
    Cyan,
    Magenta,
    Gray,
    Reset,
}

impl Color {
    pub fn code(&self) -> &str {
        match self {
            Color::Red => "\x1b[31m",
            Color::Green => "\x1b[32m",
            Color::Yellow => "\x1b[33m",
            Color::Blue => "\x1b[34m",
            Color::Cyan => "\x1b[36m",
            Color::Magenta => "\x1b[35m",
            Color::Gray => "\x1b[90m",
            Color::Reset => "\x1b[0m",
        }
    }
}

/// Print text in a specified color
pub fn print_colored(text: &str, color: Color) {
    print!("{}{}{}", color.code(), text, Color::Reset.code());
    io::stdout().flush().unwrap();
}

/// Print text in a specified color with a newline
pub fn println_colored(text: &str, color: Color) {
    println!("{}{}{}", color.code(), text, Color::Reset.code());
}

/// Print a thinking animation frame
pub fn print_thinking_animation(emoji: &str, text: &str) {
    eprint!("\r\x1B[K{}  {}", emoji, text);
    io::stderr().flush().unwrap();
}

/// Clear the current line
pub fn clear_line() {
    eprint!("\r\x1B[K");
    io::stderr().flush().unwrap();
}

/// Print a divider line
pub fn print_divider(quiet: bool) {
    if !quiet {
        eprintln!("─────────────────────────────────────────────────────");
    }
}

/// Print an error message in red
pub fn print_error(message: &str) {
    eprintln!("{}Error: {}{}", Color::Red.code(), message, Color::Reset.code());
}

/// Print a success message in green
pub fn print_success(message: &str) {
    eprintln!("{}✓ {}{}", Color::Green.code(), message, Color::Reset.code());
}
