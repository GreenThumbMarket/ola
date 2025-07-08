// Output formatting utilities

/// Enum for defining ANSI color codes
pub enum Color {
    Red,
    Green,
    Reset,
}

impl Color {
    pub fn code(&self) -> &str {
        match self {
            Color::Red => "\x1b[31m",
            Color::Green => "\x1b[32m",
            Color::Reset => "\x1b[0m",
        }
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
