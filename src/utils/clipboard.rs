// Clipboard utility for copying text to the system clipboard
use std::io::Write;
use std::process::{Command, Stdio};

/// Copy text to the system clipboard using the appropriate command for the current OS
pub fn copy_to_clipboard(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Get the operating system
    let os = std::env::consts::OS;
    
    // Use the appropriate clipboard command based on OS
    let (cmd, args) = match os {
        "macos" => ("pbcopy", vec![]),
        "linux" => ("xclip", vec!["-selection", "clipboard"]),
        "windows" => ("clip", vec![]),
        _ => {
            return Err(format!("Clipboard functionality not supported on this platform: {}", os).into());
        }
    };
    
    // Execute clipboard command
    let mut child = Command::new(cmd)
        .args(&args)
        .stdin(Stdio::piped())
        .spawn()?;
    
    {
        let stdin = child.stdin.as_mut()
            .ok_or("Failed to open clipboard command stdin")?;
        stdin.write_all(text.as_bytes())?;
    }
    
    let status = child.wait()?;
    
    if status.success() {
        Ok(())
    } else {
        Err(format!("Clipboard command failed with exit code: {:?}", status.code()).into())
    }
}

