// Anthropic API implementation
use serde_json::json;
use std::io::{BufRead, Write};
use std::time::Duration;

use super::Provider;

pub struct Anthropic {
    api_key: String,
    base_url: String,
}

impl Anthropic {
    pub fn new(api_key: &str, base_url: Option<&str>) -> Self {
        let url = base_url.unwrap_or("https://api.anthropic.com").to_string();
        Self { 
            api_key: api_key.to_string(),
            base_url: url,
        }
    }
}

impl Provider for Anthropic {
    fn send_prompt(&self, prompt: &str, model: &str, stream: bool) -> Result<String, Box<dyn std::error::Error>> {
        // Create a blocking client with timeout configuration
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(120)) // 2 minute timeout
            .build()?;
        
        // Prepare the JSON payload for Anthropic API
        let payload = json!({
            "model": model,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "max_tokens": 2048,
            "stream": stream
        });
        
        println!("Sending request to Anthropic...");
        
        // Send a POST request to the Anthropic API endpoint
        let response = client
            .post(format!("{}/v1/messages", self.base_url))
            .header("X-API-Key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()?;
        
        // Check if response is successful
        if !response.status().is_success() {
            return Err(format!("Anthropic API error: {}", response.status()).into());
        }
        
        let mut full_response = String::new();
        
        if stream {
            // Process the stream line by line
            let reader = std::io::BufReader::new(response);
            
            for line in reader.lines() {
                let line = line?;
                if line.is_empty() || line == "data: [DONE]" {
                    continue;
                }
                
                // Anthropic prefixes each line with "data: "
                if let Some(json_str) = line.strip_prefix("data: ") {
                    // Parse JSON data
                    if let Ok(json_response) = serde_json::from_str::<serde_json::Value>(json_str) {
                        // Extract content from the response
                        if let Some(delta) = json_response["delta"]["text"].as_str() {
                            print!("{}", delta);
                            std::io::stdout().flush()?;
                            full_response.push_str(delta);
                        }
                    }
                }
            }
            
            println!("\n"); // Add a newline at the end
        } else {
            // Handle non-streaming response
            let json_response: serde_json::Value = response.json()?;
            
            // Handle the Anthropic response format which has content as an array
            if let Some(content_array) = json_response["content"].as_array() {
                for item in content_array {
                    if let Some(text) = item["text"].as_str() {
                        full_response.push_str(text);
                    }
                }
            }
        }
        
        Ok(full_response)
    }
}