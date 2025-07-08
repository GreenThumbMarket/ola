// Ollama API implementation
use serde_json::json;
use std::io::{BufRead, Write};
use std::time::Duration;

use super::Provider;

pub struct Ollama {
    base_url: String,
}

impl Ollama {
    pub fn new(base_url: Option<&str>) -> Self {
        let url = base_url.unwrap_or("http://localhost:11434").to_string();
        Self { base_url: url }
    }
}

impl Provider for Ollama {
    fn send_prompt(&self, prompt: &str, model: &str, stream: bool) -> Result<String, Box<dyn std::error::Error>> {
        // Create a blocking client with timeout configuration
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(120)) // 2 minute timeout
            .build()?;
        
        // Prepare the JSON payload for Ollama API
        let payload = json!({
            "model": model,
            "prompt": prompt,
            "stream": stream,  // Enable streaming
            "options": {
                "num_predict": 2048  // Limit token output
            }
        });
        
        println!("Sending request to Ollama...");
        
        // Send a POST request to the Ollama API endpoint
        let response = client
            .post(format!("{}/api/generate", self.base_url))
            .json(&payload)
            .send()?;
        
        // Check if response is successful
        if !response.status().is_success() {
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
            let json_response: serde_json::Value = serde_json::from_str(&line)?;
            
            // Extract the response text
            if let Some(text) = json_response["response"].as_str() {
                if stream {
                    print!("{}", text);
                    std::io::stdout().flush()?;
                }
                full_response.push_str(text);
            }
        }
        
        if stream {
            println!("\n"); // Add a newline at the end
        }
        
        Ok(full_response)
    }
}