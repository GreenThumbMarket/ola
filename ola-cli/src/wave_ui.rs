/// Wave-themed TUI animations for Ola CLI
/// Provides ocean/AI-themed visual effects with blue color schemes

use console::{style, Term};
use indicatif::{ProgressBar, ProgressStyle};
use std::thread;
use std::time::Duration;

/// Ocean wave animation frames
const WAVE_FRAMES: &[&str] = &[
    "🌊 ～～～～～～～～～～～",
    "～ 🌊 ～～～～～～～～～～",
    "～～ 🌊 ～～～～～～～～～",
    "～～～ 🌊 ～～～～～～～～",
    "～～～～ 🌊 ～～～～～～～",
    "～～～～～ 🌊 ～～～～～～",
    "～～～～～～ 🌊 ～～～～～",
    "～～～～～～～ 🌊 ～～～～",
    "～～～～～～～～ 🌊 ～～～",
    "～～～～～～～～～ 🌊 ～～",
    "～～～～～～～～～～ 🌊 ～",
    "～～～～～～～～～～～ 🌊",
];

/// AI thinking animation frames with gradient effect
const THINKING_FRAMES: &[&str] = &[
    "⠋ AI thinking",
    "⠙ AI thinking",
    "⠹ AI thinking",
    "⠸ AI thinking",
    "⠼ AI thinking",
    "⠴ AI thinking",
    "⠦ AI thinking",
    "⠧ AI thinking",
    "⠇ AI thinking",
    "⠏ AI thinking",
];

pub struct WaveUI {
    term: Term,
}

impl WaveUI {
    pub fn new() -> Self {
        Self {
            term: Term::stdout(),
        }
    }

    /// Display a wave animation banner
    pub fn show_banner(&self) {
        let banner = format!(
            "\n{}\n{}\n{}\n",
            style("╔═══════════════════════════════════════╗").cyan().bold(),
            style("║     🌊  Ola - AI Wave Assistant  🌊   ║").cyan().bold(),
            style("╚═══════════════════════════════════════╝").cyan().bold()
        );
        println!("{}", banner);
    }

    /// Display animated wave loading
    pub fn show_wave_animation(&self, duration_ms: u64) {
        let iterations = duration_ms / 100;
        for i in 0..iterations {
            let frame = WAVE_FRAMES[(i as usize) % WAVE_FRAMES.len()];
            print!("\r{}", style(frame).cyan());
            std::io::Write::flush(&mut std::io::stdout()).ok();
            thread::sleep(Duration::from_millis(100));
        }
        println!();
    }

    /// Create a styled progress bar with wave theme
    pub fn create_wave_progress(&self, message: &str, len: u64) -> ProgressBar {
        let pb = ProgressBar::new(len);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.cyan} {msg} [{bar:40.cyan/blue}] {pos}/{len}")
                .unwrap()
                .progress_chars("🌊～～")
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );
        pb.set_message(message.to_string());
        pb
    }

    /// Create an infinite spinner for AI thinking
    pub fn create_thinking_spinner(&self, message: &str) -> ProgressBar {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan.bold} {msg}")
                .unwrap()
                .tick_strings(&[
                    "⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏",
                ]),
        );
        pb.set_message(style(message).cyan().to_string());
        pb.enable_steady_tick(Duration::from_millis(80));
        pb
    }

    /// Display success message with wave emoji
    pub fn show_success(&self, message: &str) {
        println!("\n{} {}", style("✓").green().bold(), style(message).cyan());
    }

    /// Display error message with wave theme
    pub fn show_error(&self, message: &str) {
        eprintln!("\n{} {}", style("✗").red().bold(), style(message).red());
    }

    /// Display info message
    pub fn show_info(&self, message: &str) {
        println!("{} {}", style("ℹ").blue().bold(), style(message).cyan());
    }

    /// Display section header with wave decoration
    pub fn show_section(&self, title: &str) {
        let decoration = "～".repeat(title.len() + 4);
        println!(
            "\n{}\n  {}  \n{}",
            style(&decoration).cyan(),
            style(title).cyan().bold(),
            style(&decoration).cyan()
        );
    }

    /// Show a pulsing wave effect
    pub fn pulse_wave(&self, iterations: usize) {
        let waves = ["～", "～～", "～～～", "～～～～", "～～～～～"];
        for _ in 0..iterations {
            for wave in &waves {
                print!("\r{} 🌊 {}", style(wave).cyan(), style(wave).cyan());
                std::io::Write::flush(&mut std::io::stdout()).ok();
                thread::sleep(Duration::from_millis(100));
            }
            for wave in waves.iter().rev() {
                print!("\r{} 🌊 {}", style(wave).cyan(), style(wave).cyan());
                std::io::Write::flush(&mut std::io::stdout()).ok();
                thread::sleep(Duration::from_millis(100));
            }
        }
        println!();
    }

    /// Display a gradient text effect (simulated with bold/dim)
    pub fn gradient_text(&self, text: &str) {
        println!("{}", style(text).cyan().bold());
    }

    /// Show recursive wave indicator for iteration levels
    pub fn show_wave_level(&self, level: u8, max_level: u8) {
        let wave_indicator = "🌊".repeat(level as usize);
        let remaining = "～".repeat((max_level - level) as usize);
        println!(
            "\n{} {} Level {}/{}",
            style(wave_indicator).cyan().bold(),
            style(remaining).cyan().dim(),
            style(level).cyan().bold(),
            style(max_level).cyan()
        );
    }

    /// Animated typing effect for AI responses
    pub fn type_text(&self, text: &str, delay_ms: u64) {
        for ch in text.chars() {
            print!("{}", style(ch).cyan());
            std::io::Write::flush(&mut std::io::stdout()).ok();
            thread::sleep(Duration::from_millis(delay_ms));
        }
        println!();
    }

    /// Clear and redraw (useful for animations)
    pub fn clear_line(&self) {
        self.term.clear_line().ok();
    }
}

impl Default for WaveUI {
    fn default() -> Self {
        Self::new()
    }
}
