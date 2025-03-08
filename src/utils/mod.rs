// Module exports for utility functions
pub mod clipboard;
pub mod output;
pub mod piping;

// Re-export frequently used utility functions
pub use clipboard::{copy_to_clipboard, is_clipboard_available};
pub use output::{print_colored, println_colored, print_error, print_success};
pub use piping::{read_from_stdin, append_to_log};
