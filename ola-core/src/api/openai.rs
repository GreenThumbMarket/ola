// OpenAI API implementation
use serde_json::json;
use std::io::{BufRead, Write};
use std::time::Duration;

use super::Provider;

pub struct OpenAI {
    api_key: String,
    base_url: String,
}

impl OpenAI {
    pub fn new(api_key: &str, base_url: Option<&str>) -> Self {
        let url = base_url.unwrap_or("https://api.openai.com").to_string();
        Self { 
            api_key: api_key.to_string(),
            base_url: url,
        }
    }
}

impl Provider for OpenAI {
    fn send_prompt(&self, prompt: &str, model: &str, stream: bool) -> Result<String, Box<dyn std::error::Error>> {
        // Create a blocking client with timeout configuration
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(120)) // 2 minute timeout
            .build()?;
        
        // Prepare the JSON payload for OpenAI API
        let payload = json!({
            "model": model,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "stream": stream
        });
        
        println!("Sending request to OpenAI...");
        
        // Send a POST request to the OpenAI API endpoint
        let response = client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()?;
        
        // Check if response is successful
        if !response.status().is_success() {
            return Err(format!("OpenAI API error: {}", response.status()).into());
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
                
                // OpenAI prefixes each line with "data: "
                if let Some(json_str) = line.strip_prefix("data: ") {
                    // Parse JSON data
                    if let Ok(json_response) = serde_json::from_str::<serde_json::Value>(json_str) {
                        // Extract content from the response
                        if let Some(content) = json_response["choices"][0]["delta"]["content"].as_str() {
                            print!("{}", content);
                            std::io::stdout().flush()?;
                            full_response.push_str(content);
                        }
                    }
                }
            }
            
            println!("\n"); // Add a newline at the end
        } else {
            // Handle non-streaming response
            let json_response: serde_json::Value = response.json()?;
            
            if let Some(content) = json_response["choices"][0]["message"]["content"].as_str() {
                full_response = content.to_string();
            }
        }
        
        Ok(full_response)
    }

    fn test_connection(&self, model: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Create a blocking client with a shorter timeout for testing
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(30)) // 30 second timeout for test
            .build()?;

        // Prepare a minimal test payload
        // Use max_completion_tokens for newer models (gpt-5, o1, etc.) and max_tokens for older ones
        let is_newer_model = model.starts_with("gpt-5") || model.starts_with("o1") ||
                            model.starts_with("o3") || model.starts_with("o4");

        let mut payload = json!({
            "model": model,
            "messages": [
                {
                    "role": "user",
                    "content": "test"
                }
            ]
        });

        if is_newer_model {
            payload["max_completion_tokens"] = json!(5);
        } else {
            payload["max_tokens"] = json!(5);
        }

        // Send a POST request to the OpenAI API endpoint
        let response = client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()?;

        // Check if response is successful
        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().unwrap_or_else(|_| "Unable to read error body".to_string());
            return Err(format!("OpenAI API error: {} - {}", status, error_body).into());
        }

        // Parse the response to ensure it's valid
        let json_response: serde_json::Value = response.json()?;

        // Check if we got a valid response structure
        if json_response["choices"].is_null() {
            return Err("Invalid response structure from OpenAI API".into());
        }

        Ok(())
    }
}