// Utilities for handling piped input/output
use std::io::{self, Read};

/// Read content from standard input (stdin)
/// Returns the content as a String or an empty string if there's an error
pub fn read_from_stdin() -> String {
    // Check if stdin has data available
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    
    let mut buffer = String::new();
    match handle.read_to_string(&mut buffer) {
        Ok(_) => buffer,
        Err(e) => {
            eprintln!("Error reading from stdin: {}", e);
            String::new()
        }
    }
}

/// Check if the program is receiving piped input
pub fn is_receiving_pipe() -> bool {
    // This is a simple check to see if stdin is connected to a terminal
    // If it's not, it might be receiving piped input
    !atty::is(atty::Stream::Stdin)
}

/// Append an entry to a log file in JSON Lines format
pub fn append_to_log(filename: &str, entry: &str) -> io::Result<()> {
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(filename)?;
    
    use std::io::Write;
    writeln!(file, "{}", entry)?;
    Ok(())
}