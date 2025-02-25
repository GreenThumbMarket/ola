use serde_json::json;
use std::fs;
use std::io::{BufRead, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

pub fn structure_reasoning(
    goals: &str,
    return_type: &str,
    warnings: &str,
    clipboard: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Build the base input data
    let mut input_data = format!(
        "Goals: {}\nReturn Type: {}\nWarnings: {}",
        goals, return_type, warnings
    );

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

    // Create a blocking client with timeout configuration
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(120)) // 2 minute timeout
        .build()?;

    // Load current configuration
    let config = crate::config::Config::load()?;
    let provider_config = config.get_active_provider().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No active provider configured. Run 'ola configure' first.",
        )
    })?;

    // Use model from config or fallback to default
    let model = provider_config
        .model
        .as_deref()
        .unwrap_or("deepseek-r1:14b");
    println!("Using model: {}", model);

    // Prepare the JSON payload for Ollama API
    let payload = json!({
        "model": model,
        "prompt": input_data,
        "stream": true,  // Enable streaming
        "options": {
            "num_predict": 2048  // Limit token output
        }
    });

    println!("Sending request to Ollama...");

    // Send a POST request to the Ollama API endpoint
    let response = match client
        .post("http://localhost:11434/api/generate")
        .json(&payload)
        .send()
    {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("Failed to send request to Ollama: {}", e);
            if e.is_timeout() {
                eprintln!("Request timed out after 120 seconds");
            }
            return Err(e.into());
        }
    };

    // Check if response is successful
    if !response.status().is_success() {
        eprintln!("Ollama returned error status: {}", response.status());
        return Err(format!("Ollama API error: {}", response.status()).into());
    }

    let mut full_response = String::new();

    // Process the stream line by line
    let reader = std::io::BufReader::new(response);
    for line in reader.lines() {
        let line = line?;
        if line.is_empty() {
            continue;
        }

        // Parse each line as JSON
        let json_response: serde_json::Value = match serde_json::from_str(&line) {
            Ok(json) => json,
            Err(e) => {
                eprintln!("Failed to parse JSON response: {}", e);
                eprintln!("Raw line: {}", line);
                continue;
            }
        };

        // Extract and print the response text
        if let Some(text) = json_response["response"].as_str() {
            print!("{}", text);
            std::io::stdout().flush()?;
            full_response.push_str(text);
        }
    }

    println!("\n"); // Add a newline at the end

    // Copy to clipboard only if the clipboard flag is set
    if clipboard {
        // Copy the complete response to the clipboard using pbcopy
        let mut pbcopy = Command::new("pbcopy")
            .stdin(Stdio::piped())
            .spawn()
            .expect("Failed to start pbcopy");
        {
            let stdin = pbcopy.stdin.as_mut().expect("Failed to open pbcopy stdin");
            stdin.write_all(full_response.as_bytes())?;
        }
        pbcopy.wait()?;
        println!("Successfully processed response and copied to clipboard");
    } else {
        println!("Successfully processed response");
    }
    
    Ok(())
}
