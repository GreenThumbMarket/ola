use std::io::Write;
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
        "stream": false,  // Get complete response rather than stream
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

    // Get response as text
    let response_text = response.text()?;

    // Parse the JSON response
    let json_response: serde_json::Value = match serde_json::from_str(&response_text) {
        Ok(json) => json,
        Err(e) => {
            eprintln!("Failed to parse JSON response: {}", e);
            eprintln!("Raw response: {}", response_text);
            return Err(e.into());
        }
    };

    // Extract the response text
    let text_output = json_response["response"]
        .as_str()
        .ok_or("Missing response text in the JSON response")?;

    // Copy the text to the clipboard using pbcopy
    let mut pbcopy = Command::new("pbcopy")
        .stdin(Stdio::piped())
        .spawn()
        .expect("Failed to start pbcopy");
    {
        let stdin = pbcopy.stdin.as_mut().expect("Failed to open pbcopy stdin");
        stdin.write_all(text_output.as_bytes())?;
    }
    pbcopy.wait()?;

    println!("Successfully processed response and copied to clipboard");
    Ok(())
}