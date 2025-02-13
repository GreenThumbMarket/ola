use std::io::{Write, BufRead};
use std::time::Duration;
use serde_json::json;
use std::process::{Command, Stdio};

pub fn structure_reasoning(goals: &str, return_type: &str, warnings: &str) -> Result<(), Box<dyn std::error::Error>> {
    let input_data = format!("Goals: {}\nReturn Type: {}\nWarnings: {}", goals, return_type, warnings);

    // Create a blocking client with timeout configuration
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(120))  // 2 minute timeout
        .build()?;

    // Prepare the JSON payload for Ollama API
    let payload = json!({
        "model": "deepseek-r1:14b",
        "prompt": input_data,
        "stream": true,  // Enable streaming
        "options": {
            "num_predict": 2048,  // Limit token output
        }
    });

    println!("Sending request to Ollama...");
    
    // Send a POST request to the Ollama API endpoint
    let response = match client
        .post("http://localhost:11434/api/generate")
        .json(&payload)
        .send() {
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
    Ok(())
}