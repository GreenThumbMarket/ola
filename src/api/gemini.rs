// Google Gemini API implementation
use serde_json::json;
use std::io::{BufRead, Write};
use std::time::Duration;

use super::Provider;

pub struct Gemini {
    api_key: String,
    base_url: String,
}

impl Gemini {
    pub fn new(api_key: &str, base_url: Option<&str>) -> Self {
        let url = base_url.unwrap_or("https://generativelanguage.googleapis.com").to_string();
        Self { 
            api_key: api_key.to_string(),
            base_url: url,
        }
    }
}

impl Provider for Gemini {
    fn send_prompt(&self, prompt: &str, model: &str, stream: bool) -> Result<String, Box<dyn std::error::Error>> {
        // Create a blocking client with timeout configuration
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(120)) // 2 minute timeout
            .build()?;
        
        // Create the API endpoint with model and API key
        let api_url = format!("{}/v1beta/models/{}:generateContent?key={}", 
            self.base_url, model, self.api_key);
        
        // Prepare the JSON payload for Gemini API
        let payload = json!({
            "contents": [
                {
                    "role": "user",
                    "parts": [
                        {
                            "text": prompt
                        }
                    ]
                }
            ],
            "generationConfig": {
                "temperature": 0.7,
                "maxOutputTokens": 2048
            }
        });
        
        println!("Sending request to Google Gemini...");
        
        // Send a POST request to the Gemini API endpoint
        let response = client
            .post(api_url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()?;
        
        // Check if response is successful
        if !response.status().is_success() {
            return Err(format!("Gemini API error: {}", response.status()).into());
        }
        
        let json_response: serde_json::Value = response.json()?;
        let mut full_response = String::new();
        
        // Extract text from response
        if let Some(candidates) = json_response["candidates"].as_array() {
            if let Some(candidate) = candidates.first() {
                if let Some(content) = candidate["content"].as_object() {
                    if let Some(parts) = content["parts"].as_array() {
                        if stream {
                            // In streaming mode, print each part as soon as it's processed
                            for part in parts {
                                if let Some(text) = part["text"].as_str() {
                                    println!("{}", text);
                                    full_response.push_str(text);
                                }
                            }
                        } else {
                            // Accumulate all text and return at once
                            for part in parts {
                                if let Some(text) = part["text"].as_str() {
                                    full_response.push_str(text);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(full_response)
    }
    
    fn get_provider_name(&self) -> &str {
        "Gemini"
    }
}
