// Prompt handling logic module
use serde_json::json;
use std::io::{BufRead, Write};
use std::path::Path;
use std::fs;
use std::time::Duration;
use regex::Regex;

use crate::api::{create_api_client_from_config, format_prompt};
use crate::utils::{clipboard, output, piping};

// Structure to hold the result of a prompt for testing
#[derive(Debug)]
pub struct PromptResult {
    pub content: String,
    pub model: String,
}

/// Main function for structured reasoning with <think> blocks
pub fn structure_reasoning(
    goals: &str,
    return_type: &str,
    warnings: &str,
    clipboard: bool,
    context: Option<&str>,
    no_thinking: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Try to load settings
    let settings = crate::settings::Settings::load().unwrap_or_default();
    
    // Format the prompt with goals, return type, warnings, and optional context
    let mut input_data = format_prompt(goals, return_type, warnings, context);
    
    // Read and append hints if available
    append_hints_if_available(&mut input_data)?;
    
    // Load current configuration and create API client
    let api_client = create_api_client_from_config()?;
    
    // Use model from config, settings, or fallback to default
    let config = crate::config::Config::load()?;
    let provider_config = config.get_active_provider().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No active provider configured. Run 'ola configure' first.",
        )
    })?;
    
    let model = provider_config
        .model
        .as_deref()
        .unwrap_or(&settings.default_model);
    println!("Using model: {}", model);
    
    // Stream the response
    let response = stream_response(&api_client, &input_data, model, no_thinking)?;
    
    // Handle clipboard copy if requested
    if clipboard {
        match clipboard::copy_to_clipboard(&response) {
            Ok(_) => output::print_success("Response copied to clipboard"),
            Err(e) => output::print_error(&format!("Failed to copy to clipboard: {}", e))
        }
    }
    
    // Log session if enabled in settings
    if settings.behavior.enable_logging {
        log_session(goals, return_type, warnings, model, &response)?;
    }
    
    Ok(())
}

/// Stream raw prompt without structured reasoning
pub fn stream_non_think(
    prompt: &str,
    clipboard: bool,
    context: Option<&str>,
    filter_thinking: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Try to load settings
    let settings = crate::settings::Settings::load().unwrap_or_default();
    
    // Format the prompt with optional context
    let mut input_data = if let Some(ctx) = context {
        format!("{}\nContext: {}", prompt, ctx)
    } else {
        prompt.to_string()
    };
    
    // Read and append hints if available
    append_hints_if_available(&mut input_data)?;
    
    // Create API client
    let api_client = create_api_client_from_config()?;
    
    // Get model information
    let config = crate::config::Config::load()?;
    let provider_config = config.get_active_provider().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No active provider configured. Run 'ola configure' first.",
        )
    })?;
    
    let model = provider_config
        .model
        .as_deref()
        .unwrap_or(&settings.default_model);
    println!("Using model: {}", model);
    
    // Stream the response
    let response = stream_response(&api_client, &input_data, model, filter_thinking)?;
    
    // Handle clipboard copy if requested
    if clipboard {
        match clipboard::copy_to_clipboard(&response) {
            Ok(_) => output::print_success("Response copied to clipboard"),
            Err(e) => output::print_error(&format!("Failed to copy to clipboard: {}", e))
        }
    }
    
    // Log session if enabled in settings
    if settings.behavior.enable_logging {
        let log_entry = json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "prompt": prompt,
            "model": model,
            "output_length": response.len(),
        });
        
        if let Err(e) = piping::append_to_log(&settings.behavior.log_file, &log_entry.to_string()) {
            eprintln!("Failed to log session: {}", e);
        }
    }
    
    Ok(())
}

// Helper function to stream response with thinking block filtering if needed
fn stream_response(
    api_client: &crate::api::ApiClient,
    prompt: &str,
    model: &str,
    filter_thinking: bool
) -> Result<String, Box<dyn std::error::Error>> {
    // Get the raw response
    let response = api_client.stream_prompt(prompt, model)?;
    
    // If we need to filter thinking blocks, process the response
    if filter_thinking {
        // Use regex to remove thinking blocks
        let re = Regex::new(r"<think>.*?</think>")?;
        let filtered_response = re.replace_all(&response, "").to_string();
        Ok(filtered_response)
    } else {
        Ok(response)
    }
}

// Helper function to read and append hints from .olaHints file
fn append_hints_if_available(input_data: &mut String) -> Result<(), Box<dyn std::error::Error>> {
    // Try to read hints from a local .olaHints file; if not found, fallback to global hints
    let mut hints = String::new();
    
    // Check local file .olaHints in the current directory
    if Path::new("./.olaHints").exists() {
        hints = fs::read_to_string("./.olaHints")?;
    } else {
        // Fallback to global hints in ~/.ola-hints/olaHints
        if let Ok(home) = std::env::var("HOME") {
            let global_path = format!("{}/.ola-hints/olaHints", home);
            if Path::new(&global_path).exists() {
                hints = fs::read_to_string(global_path)?;
            }
        }
    }

    // If hints were found, append them to the input data
    if !hints.is_empty() {
        input_data.push_str(&format!("\nHINTS: {}", hints));
    }
    
    Ok(())
}

// Helper function to log session information
fn log_session(
    goals: &str,
    return_type: &str,
    warnings: &str,
    model: &str,
    response: &str
) -> Result<(), Box<dyn std::error::Error>> {
    let settings = crate::settings::Settings::load().unwrap_or_default();
    
    // Get recursion wave number if present
    let wave_number = std::env::var("OLA_RECURSION_WAVE")
        .ok()
        .and_then(|s| s.parse::<u8>().ok());
    
    // Build log entry with optional recursion information
    let mut log_entry = json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "goals": goals,
        "return_format": return_type,
        "warnings": warnings,
        "model": model,
        "output_length": response.len(),
    });
    
    // Add recursion wave info if available
    if let Some(wave) = wave_number {
        log_entry["recursion_wave"] = json!(wave);
    }
    
    piping::append_to_log(&settings.behavior.log_file, &log_entry.to_string())?;
    Ok(())
}

// Testing functions
#[cfg(test)]
pub fn structure_reasoning_test(
    goals: &str,
    return_type: &str,
    warnings: &str,
    clipboard: bool,
    context: Option<&str>,
    no_thinking: bool,
) -> Result<PromptResult, Box<dyn std::error::Error>> {
    // This is a testable version that returns the response instead of printing it
    let settings = crate::settings::Settings::load().unwrap_or_default();
    
    // Build the input data with optional context
    let input_data = if let Some(ctx) = context {
        format!(
            "Goals: {}\nReturn Format: {}\nWarnings: {}\nContext: {}",
            goals, return_type, warnings, ctx
        )
    } else {
        format!(
            "Goals: {}\nReturn Format: {}\nWarnings: {}",
            goals, return_type, warnings
        )
    };

    // Load configuration
    let config = crate::config::Config::load()?;
    let provider_config = config.get_active_provider().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No active provider configured.",
        )
    })?;

    // Use model from config or settings
    let model = provider_config
        .model
        .as_deref()
        .unwrap_or(&settings.default_model);
    
    // Here's where we would mock the API call in tests
    // For testing, we'll just return the input as the response
    Ok(PromptResult {
        content: format!("This is a mocked response from the {} API.", provider_config.provider),
        model: model.to_string(),
    })
}

#[cfg(test)]
pub fn stream_non_think_test(
    prompt: &str,
    clipboard: bool,
    context: Option<&str>,
    filter_thinking: bool,
) -> Result<PromptResult, Box<dyn std::error::Error>> {
    // This is a testable version that returns the response instead of printing it
    let settings = crate::settings::Settings::load().unwrap_or_default();
    
    // Build the input data with optional context
    let input_data = if let Some(ctx) = context {
        format!("{}\nContext: {}", prompt, ctx)
    } else {
        prompt.to_string()
    };

    // Load configuration
    let config = crate::config::Config::load()?;
    let provider_config = config.get_active_provider().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No active provider configured.",
        )
    })?;

    // Use model from config or settings
    let model = provider_config
        .model
        .as_deref()
        .unwrap_or(&settings.default_model);
    
    // Here's where we would mock the API call in tests
    // For testing, we'll just return the input as the response
    Ok(PromptResult {
        content: format!("This is a mocked response from the {} API.", provider_config.provider),
        model: model.to_string(),
    })
}