// NeoVim integration utility functions
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tempfile::NamedTempFile;

/// Check if NeoVim is available on the system
pub fn is_nvim_available(nvim_path: &str) -> bool {
    let os = std::env::consts::OS;
    
    // Construct the command to check if NeoVim is installed
    let (cmd, args) = match os {
        "windows" => ("where", vec![nvim_path.to_string()]),
        _ => ("which", vec![nvim_path.to_string()]),
    };
    
    // Execute the command
    match Command::new(cmd).args(&args).stdout(Stdio::null()).status() {
        Ok(status) => status.success(),
        Err(_) => false,
    }
}

/// Open the response in NeoVim for editing
pub fn open_in_nvim(
    content: &str,
    nvim_path: &str,
    nvim_args: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    // Create a temporary file to hold the content
    let mut temp_file = NamedTempFile::new()?;
    
    // Write the content to the temporary file
    temp_file.write_all(content.as_bytes())?;
    
    // Get the path to the temporary file
    let temp_path = temp_file.path().to_str().ok_or("Failed to get temporary file path")?;
    
    // Get the operating system
    let os = std::env::consts::OS;
    
    // Build the command to execute NeoVim
    let mut command = Command::new(nvim_path);
    
    // Add any custom arguments
    if !nvim_args.is_empty() {
        command.args(nvim_args);
    }
    
    // Add the path to the temporary file
    command.arg(temp_path);
    
    // Execute the command with appropriate shell and terminal settings
    let status = if os == "windows" {
        command.status()?
    } else {
        // On Unix-like systems, we want to ensure the terminal is properly handled
        command.status()?
    };
    
    if status.success() {
        // Read back the potentially edited content from the temp file
        let edited_content = fs::read_to_string(temp_path)?;
        
        // Output the edited content to stdout for potential piping
        println!("{}", edited_content);
        
        Ok(())
    } else {
        Err(format!("NeoVim exited with non-zero status: {:?}", status.code()).into())
    }
}

/// Get path to NeoVim executable from settings or use default
pub fn get_nvim_path() -> String {
    match crate::settings::Settings::load() {
        Ok(settings) => settings.behavior.nvim.path,
        Err(_) => "nvim".to_string(),
    }
}

/// Determine if NeoVim should be used based on settings and command line flags
pub fn should_use_nvim(cli_nvim: bool, cli_no_nvim: bool) -> bool {
    // Command line flags take precedence over settings
    if cli_nvim {
        return true;
    }
    
    if cli_no_nvim {
        return false;
    }
    
    // Otherwise, use the setting from the config
    match crate::settings::Settings::load() {
        Ok(settings) => settings.behavior.nvim.enabled,
        Err(_) => false, // Default to false if settings can't be loaded
    }
}