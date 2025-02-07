use std::io::{Write, Result as IoResult};
use serde_json::json;
use std::process::{Command, Stdio};

// This version sends a POST request, parses the JSON response to extract only choices[0].text,
// and copies that text to the clipboard using pbcopy.

pub fn structure_reasoning(goals: &str, return_type: &str, warnings: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("In Here");
    let input_data = format!("Goals: {}\nReturn Type: {}\nWarnings: {}", goals, return_type, warnings);

    // Create a blocking client
    let client = reqwest::blocking::Client::new();

    // Prepare the JSON payload
    let payload = json!({
        "prompt": input_data,
        "model": "deepseek-r1:14b"
    });

    // Send a POST request to the Ollama completions endpoint.
    // Note: Using port 11434 as per the curl script in ../olaPipe/olapipe.sh
    let response_text = client
        .post("http://localhost:11434/v1/completions")
        .json(&payload)
        .send()?
        .text()?;

    // Parse the JSON response
    let json_response: serde_json::Value = serde_json::from_str(&response_text)?;

    // Extract only choices[0].text
    let text_output = json_response.get("choices")
        .and_then(|choices| choices.get(0))
        .and_then(|choice| choice.get("text"))
        .and_then(|t| t.as_str())
        .ok_or("Missing choices[0].text in the response JSON")?;

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

    Ok(())
}
