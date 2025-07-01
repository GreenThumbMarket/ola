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
    // Bright colors
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightCyan,
    BrightMagenta,
    BrightWhite,
    // RGB colors for more vibrant effects
    DeepSkyBlue,
    Turquoise,
    SeaGreen,
    Orange,
    Purple,
    Pink,
    Lime,
    Gold,
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
            // Bright colors
            Color::BrightRed => "\x1b[91m",
            Color::BrightGreen => "\x1b[92m",
            Color::BrightYellow => "\x1b[93m",
            Color::BrightBlue => "\x1b[94m",
            Color::BrightCyan => "\x1b[96m",
            Color::BrightMagenta => "\x1b[95m",
            Color::BrightWhite => "\x1b[97m",
            // RGB colors for vibrant effects
            Color::DeepSkyBlue => "\x1b[38;5;39m",
            Color::Turquoise => "\x1b[38;5;45m",
            Color::SeaGreen => "\x1b[38;5;23m",
            Color::Orange => "\x1b[38;5;208m",
            Color::Purple => "\x1b[38;5;129m",
            Color::Pink => "\x1b[38;5;206m",
            Color::Lime => "\x1b[38;5;154m",
            Color::Gold => "\x1b[38;5;220m",
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

/// Print a rainbow gradient text
pub fn print_rainbow(text: &str) {
    let colors = [
        Color::BrightRed,
        Color::Orange,
        Color::BrightYellow,
        Color::BrightGreen,
        Color::BrightCyan,
        Color::BrightBlue,
        Color::BrightMagenta,
    ];
    
    for (i, ch) in text.chars().enumerate() {
        let color = &colors[i % colors.len()];
        print!("{}{}", color.code(), ch);
    }
    println!("{}", Color::Reset.code());
}

/// Print text with a pulsing effect using different intensities
pub fn print_pulsing(text: &str, color: Color) {
    // Create a pulsing effect by alternating between normal and bright versions
    let bright_code = match color {
        Color::Red => Color::BrightRed.code(),
        Color::Green => Color::BrightGreen.code(),
        Color::Blue => Color::BrightBlue.code(),
        Color::Cyan => Color::BrightCyan.code(),
        Color::Magenta => Color::BrightMagenta.code(),
        Color::Yellow => Color::BrightYellow.code(),
        _ => color.code(),
    };
    
    for (i, ch) in text.chars().enumerate() {
        if i % 2 == 0 {
            print!("{}{}", color.code(), ch);
        } else {
            print!("{}{}", bright_code, ch);
        }
    }
    println!("{}", Color::Reset.code());
}

/// Print a stylized banner with borders
pub fn print_banner(text: &str, color: Color) {
    let width = text.len() + 4;
    let border = "═".repeat(width);
    
    println!("{}╔{}╗{}", color.code(), border, Color::Reset.code());
    println!("{}║  {}  ║{}", color.code(), text, Color::Reset.code());
    println!("{}╚{}╝{}", color.code(), border, Color::Reset.code());
}

/// Print an animated spinner
pub fn print_spinner_frame(frame: usize, message: &str) {
    let spinners = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let spinner = spinners[frame % spinners.len()];
    eprint!("\r\x1B[K{}{} {} {}", Color::BrightCyan.code(), spinner, message, Color::Reset.code());
    io::stderr().flush().unwrap();
}

/// Print wave animation
pub fn print_wave_animation(frame: usize, text: &str) {
    let waves = ["🌊", "🌊🌊", "🌊🌊🌊", "🌊🌊", "🌊"];
    let wave = waves[frame % waves.len()];
    eprint!("\r\x1B[K{}{} {}{}", Color::DeepSkyBlue.code(), wave, text, Color::Reset.code());
    io::stderr().flush().unwrap();
}

/// Print progress bar
pub fn print_progress_bar(current: usize, total: usize, width: usize) {
    let progress = (current * width) / total;
    let bar: String = "█".repeat(progress) + &"░".repeat(width - progress);
    
    let percentage = (current * 100) / total;
    print!("\r{}{} {}%{}", 
           Color::BrightGreen.code(), 
           bar, 
           percentage, 
           Color::Reset.code());
    io::stdout().flush().unwrap();
}

/// Display the OLA ASCII art with colors
pub fn display_ola_logo() {
    let ascii_art = include_str!("../ascii.txt");
    let lines: Vec<&str> = ascii_art.lines().collect();
    
    // Print the main OLA text in rainbow
    for (i, line) in lines.iter().take(8).enumerate() {
        if i == 0 || line.trim().is_empty() {
            println!("{}", line);
        } else {
            print_rainbow(line);
        }
    }
    
    // Print the braille art in ocean colors
    for line in lines.iter().skip(8).take(15) {
        println_colored(line, Color::DeepSkyBlue);
    }
    
    // Print the wave lines in animated style
    for line in lines.iter().skip(23) {
        if line.contains("~") {
            println_colored(line, Color::Turquoise);
        } else {
            println!("{}", line);
        }
    }
}

/// Print a fancy startup animation
pub fn startup_animation() {
    display_ola_logo();
    println!();
    print_rainbow("🌊 Welcome to Ola - Your Ocean of AI Possibilities! 🌊");
    println!();
}
