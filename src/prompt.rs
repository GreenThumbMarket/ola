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
    context: Option<&str>,
    no_thinking: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Try to load settings
    let settings = crate::settings::Settings::load().unwrap_or_default();
    
    // Use custom prefixes from settings if available
    let goals_prefix = &settings.prompt_template.goals_prefix;
    let return_format_prefix = &settings.prompt_template.return_format_prefix;
    let warnings_prefix = &settings.prompt_template.warnings_prefix;
    
    // Build the base input data with optional context
    let mut input_data = if let Some(ctx) = context {
        format!(
            "{}{}\n{}{}\n{}{}\nContext: {}",
            goals_prefix, goals, 
            return_format_prefix, return_type, 
            warnings_prefix, warnings, 
            ctx
        )
    } else {
        format!(
            "{}{}\n{}{}\n{}{}",
            goals_prefix, goals, 
            return_format_prefix, return_type, 
            warnings_prefix, warnings
        )
    };

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

    // Use model from config, settings, or fallback to default
    let model = provider_config
        .model
        .as_deref()
        .unwrap_or(&settings.default_model);
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
    
    // Variables for filtering thinking blocks
    let mut in_thinking_block = false;
    let mut thinking_animation_counter = 0;
    let thinking_emojis = &settings.behavior.thinking_animation.emojis;
    let thinking_text = &settings.behavior.thinking_animation.text;
    
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

        // Extract the response text
        if let Some(text) = json_response["response"].as_str() {
            if no_thinking {
                // Check for <think> and </think> tags
                if text.contains("<think>") {
                    in_thinking_block = true;
                    // Clear current line and print thinking animation
                    eprint!("\r\x1B[K{}  {}", 
                        &thinking_emojis[thinking_animation_counter % thinking_emojis.len()], 
                        thinking_text);
                    std::io::stderr().flush()?;
                    thinking_animation_counter += 1;
                } else if text.contains("</think>") {
                    in_thinking_block = false;
                    // Clear the thinking animation line
                    eprint!("\r\x1B[K");
                    std::io::stderr().flush()?;
                } else if in_thinking_block {
                    // For text inside thinking block, just update animation frame
                    if thinking_animation_counter % 4 == 0 {
                        eprint!("\r\x1B[K{}  {}", 
                            &thinking_emojis[thinking_animation_counter % thinking_emojis.len()], 
                            thinking_text);
                        std::io::stderr().flush()?;
                        thinking_animation_counter += 1;
                    }
                } else {
                    // Outside thinking block, print normally
                    print!("{}", text);
                    std::io::stdout().flush()?;
                }
            } else {
                // No filtering, print everything 
                print!("{}", text);
                std::io::stdout().flush()?;
            }
            // Always add to full response
            full_response.push_str(text);
        }
    }

    println!("\n"); // Add a newline at the end

    // Copy to clipboard only if the clipboard flag is set
    if clipboard {
        // Get the operating system
        let os = std::env::consts::OS;
        
        // Use the appropriate clipboard command based on OS
        let (cmd, args) = match os {
            "macos" => ("pbcopy", vec![]),
            "linux" => ("xclip", vec!["-selection", "clipboard"]),
            "windows" => ("clip", vec![]),
            _ => {
                eprintln!("Clipboard functionality not supported on this platform: {}", os);
                return Ok(());
            }
        };
        
        // Execute clipboard command
        let status = match Command::new(cmd)
            .args(&args)
            .stdin(Stdio::piped())
            .spawn() {
                Ok(mut child) => {
                    {
                        let stdin = child.stdin.as_mut()
                            .expect("Failed to open clipboard command stdin");
                        stdin.write_all(full_response.as_bytes())?;
                    }
                    child.wait()
                },
                Err(e) => {
                    eprintln!("Failed to start clipboard command: {}. Error: {}", cmd, e);
                    return Ok(());
                }
            };
            
        if let Ok(exit_status) = status {
            if exit_status.success() {
                eprintln!("Successfully processed response and copied to clipboard");
            } else {
                eprintln!("Clipboard command failed with exit code: {:?}", exit_status.code());
            }
        }
    } else {
        eprintln!("Successfully processed response");
    }
    
    // Log session if enabled in settings
    if settings.behavior.enable_logging {
        // Get recursion wave number if present
        let wave_number = std::env::var("OLA_RECURSION_WAVE").ok().and_then(|s| s.parse::<u8>().ok());
        
        // Build log entry with optional recursion information
        let mut log_entry = json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "goals": goals,
            "return_format": return_type,
            "warnings": warnings,
            "model": model,
            "output_length": full_response.len(),
        });
        
        // Add recursion wave info if available
        if let Some(wave) = wave_number {
            log_entry["recursion_wave"] = json!(wave);
        }
        
        if let Err(e) = append_to_log(&settings.behavior.log_file, &log_entry.to_string()) {
            eprintln!("Failed to log session: {}", e);
        }
    }
    
    Ok(())
}

pub fn stream_non_think(
    prompt: &str,
    clipboard: bool,
    context: Option<&str>,
    filter_thinking: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Try to load settings
    let settings = crate::settings::Settings::load().unwrap_or_default();
    
    // Build the input data with optional context
    let mut input_data = if let Some(ctx) = context {
        format!(
            "{}\nContext: {}",
            prompt, ctx
        )
    } else {
        prompt.to_string()
    };

    // Try to read hints from a local .olaHints file; if not found, fallback to global hints
    let mut hints = String::new();
    if Path::new("./.olaHints").exists() {
        hints = fs::read_to_string("./.olaHints")?;
    } else {
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

    // Use model from config, settings, or fallback to default
    let model = provider_config
        .model
        .as_deref()
        .unwrap_or(&settings.default_model);
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
    
    // Variables for filtering thinking blocks
    let mut in_thinking_block = false;
    let mut thinking_animation_counter = 0;
    let thinking_emojis = &settings.behavior.thinking_animation.emojis;
    let thinking_text = &settings.behavior.thinking_animation.text;
    
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

        // Extract the response text
        if let Some(text) = json_response["response"].as_str() {
            if filter_thinking {
                // Check for <think> and </think> tags
                if text.contains("<think>") {
                    in_thinking_block = true;
                    // Clear current line and print thinking animation
                    eprint!("\r\x1B[K{}  {}", 
                        &thinking_emojis[thinking_animation_counter % thinking_emojis.len()], 
                        thinking_text);
                    std::io::stderr().flush()?;
                    thinking_animation_counter += 1;
                } else if text.contains("</think>") {
                    in_thinking_block = false;
                    // Clear the thinking animation line
                    eprint!("\r\x1B[K");
                    std::io::stderr().flush()?;
                } else if in_thinking_block {
                    // For text inside thinking block, just update animation frame
                    if thinking_animation_counter % 4 == 0 {
                        eprint!("\r\x1B[K{}  {}", 
                            &thinking_emojis[thinking_animation_counter % thinking_emojis.len()], 
                            thinking_text);
                        std::io::stderr().flush()?;
                        thinking_animation_counter += 1;
                    }
                } else {
                    // Outside thinking block, print normally
                    print!("{}", text);
                    std::io::stdout().flush()?;
                }
                
                // Always add to full response
                full_response.push_str(text);
            } else {
                // No filtering, print everything
                print!("{}", text);
                std::io::stdout().flush()?;
                full_response.push_str(text);
            }
        }
    }

    println!("\n"); // Add a newline at the end

    // Copy to clipboard only if the clipboard flag is set
    if clipboard {
        // Get the operating system
        let os = std::env::consts::OS;
        
        // Use the appropriate clipboard command based on OS
        let (cmd, args) = match os {
            "macos" => ("pbcopy", vec![]),
            "linux" => ("xclip", vec!["-selection", "clipboard"]),
            "windows" => ("clip", vec![]),
            _ => {
                eprintln!("Clipboard functionality not supported on this platform: {}", os);
                return Ok(());
            }
        };
        
        // Execute clipboard command
        let status = match Command::new(cmd)
            .args(&args)
            .stdin(Stdio::piped())
            .spawn() {
                Ok(mut child) => {
                    {
                        let stdin = child.stdin.as_mut()
                            .expect("Failed to open clipboard command stdin");
                        stdin.write_all(full_response.as_bytes())?;
                    }
                    child.wait()
                },
                Err(e) => {
                    eprintln!("Failed to start clipboard command: {}. Error: {}", cmd, e);
                    return Ok(());
                }
            };
            
        if let Ok(exit_status) = status {
            if exit_status.success() {
                eprintln!("Successfully processed response and copied to clipboard");
            } else {
                eprintln!("Clipboard command failed with exit code: {:?}", exit_status.code());
            }
        }
    } else {
        eprintln!("Successfully processed response");
    }
    
    // Log session if enabled in settings
    if settings.behavior.enable_logging {
        // Get recursion wave number if present
        let wave_number = std::env::var("OLA_RECURSION_WAVE").ok().and_then(|s| s.parse::<u8>().ok());
        
        // Build log entry with optional recursion information
        let mut log_entry = json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "prompt": prompt,
            "model": model,
            "output_length": full_response.len(),
        });
        
        // Add recursion wave info if available
        if let Some(wave) = wave_number {
            log_entry["recursion_wave"] = json!(wave);
        }
        
        if let Err(e) = append_to_log(&settings.behavior.log_file, &log_entry.to_string()) {
            eprintln!("Failed to log session: {}", e);
        }
    }
    
    Ok(())
}

fn append_to_log(filename: &str, entry: &str) -> std::io::Result<()> {
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(filename)?;
    writeln!(file, "{}", entry)?;
    Ok(())
}