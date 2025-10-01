// Context file management module
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};

use crate::utils::output;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ContextConfig {
    pub files: Vec<String>,
}

impl ContextConfig {
    /// Get the path to the context config file
    fn config_path() -> Result<PathBuf> {
        let current_dir = std::env::current_dir()?;
        Ok(current_dir.join(".ola-context.json"))
    }

    /// Load context configuration from file
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path)
            .context("Failed to read context configuration file")?;

        let config: ContextConfig = serde_json::from_str(&content)
            .context("Failed to parse context configuration")?;

        Ok(config)
    }

    /// Save context configuration to file
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        let content = serde_json::to_string_pretty(self)
            .context("Failed to serialize context configuration")?;

        fs::write(&path, content)
            .context("Failed to write context configuration file")?;

        Ok(())
    }

    /// Add a file to the context list
    pub fn add_file(&mut self, file_path: &str) -> Result<()> {
        // Check if file exists
        let path = Path::new(file_path);
        if !path.exists() {
            anyhow::bail!("File does not exist: {}", file_path);
        }

        // Check if it's a file (not a directory)
        if !path.is_file() {
            anyhow::bail!("Path is not a file: {}", file_path);
        }

        // Check if already in the list
        if self.files.contains(&file_path.to_string()) {
            anyhow::bail!("File already in context list: {}", file_path);
        }

        self.files.push(file_path.to_string());
        self.save()?;

        Ok(())
    }

    /// Remove a file from the context list
    pub fn remove_file(&mut self, file_path: &str) -> Result<()> {
        let original_len = self.files.len();
        self.files.retain(|f| f != file_path);

        if self.files.len() == original_len {
            anyhow::bail!("File not found in context list: {}", file_path);
        }

        self.save()?;
        Ok(())
    }

    /// List all files in the context
    pub fn list_files(&self) {
        if self.files.is_empty() {
            output::println_colored("No context files configured.", output::Color::BrightYellow);
            println!("\nTip: Add files with 'ola context add <file>'");
            return;
        }

        output::println_colored("📄 Context Files:", output::Color::BrightCyan);
        println!();

        for (index, file) in self.files.iter().enumerate() {
            let path = Path::new(file);
            let exists = path.exists();

            if exists {
                print!("  {}. {} ", index + 1, file);
                output::println_colored("✓", output::Color::BrightGreen);
            } else {
                print!("  {}. {} ", index + 1, file);
                output::println_colored("✗ (missing)", output::Color::BrightRed);
            }
        }
    }

    /// Read all context files and return their contents
    pub fn read_all_files(&self) -> Result<Vec<(String, String)>> {
        let mut results = Vec::new();

        for file_path in &self.files {
            let path = Path::new(file_path);

            if !path.exists() {
                eprintln!("Warning: Context file not found, skipping: {}", file_path);
                continue;
            }

            match fs::read_to_string(path) {
                Ok(content) => {
                    // Limit file size to avoid token overflow (max 5000 chars per file)
                    let content = if content.len() > 5000 {
                        format!("{}...\n[Content truncated - file too large]", &content[..5000])
                    } else {
                        content
                    };

                    results.push((file_path.clone(), content));
                }
                Err(e) => {
                    eprintln!("Warning: Failed to read {}: {}", file_path, e);
                }
            }
        }

        Ok(results)
    }

    /// Clear all context files
    pub fn clear(&mut self) -> Result<()> {
        self.files.clear();
        self.save()?;
        Ok(())
    }
}

/// Add a file to the context list
pub fn add_file(file_path: &str) -> Result<()> {
    let mut config = ContextConfig::load()?;
    config.add_file(file_path)?;

    output::print_success(&format!("Added '{}' to context", file_path));
    Ok(())
}

/// Remove a file from the context list
pub fn remove_file(file_path: &str) -> Result<()> {
    let mut config = ContextConfig::load()?;
    config.remove_file(file_path)?;

    output::print_success(&format!("Removed '{}' from context", file_path));
    Ok(())
}

/// List all context files
pub fn list_files() -> Result<()> {
    let config = ContextConfig::load()?;
    config.list_files();
    Ok(())
}

/// Clear all context files
pub fn clear_files() -> Result<()> {
    let mut config = ContextConfig::load()?;
    let count = config.files.len();
    config.clear()?;

    output::print_success(&format!("Cleared {} context file(s)", count));
    Ok(())
}
