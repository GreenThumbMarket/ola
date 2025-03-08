use mockito::{Mock, Server};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use tempfile::{tempdir, TempDir};

// Common test utilities

/// Create a mock for OpenAI API
pub fn mock_openai_api() -> Mock {
    let mut server = Server::new();
    server.mock("POST", "/v1/chat/completions")
        .with_header("content-type", "application/json")
        .with_body(r#"{
            "id": "chatcmpl-123",
            "object": "chat.completion",
            "created": 1677652288,
            "model": "gpt-4",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "This is a mocked response from the OpenAI API."
                },
                "finish_reason": "stop"
            }]
        }"#)
}

/// Create a mock for Anthropic API
pub fn mock_anthropic_api() -> Mock {
    let mut server = Server::new();
    server.mock("POST", "/v1/messages")
        .with_header("content-type", "application/json")
        .with_body(r#"{
            "id": "msg_1234567890",
            "type": "message",
            "role": "assistant",
            "content": [
                {
                    "type": "text",
                    "text": "This is a mocked response from the Anthropic API."
                }
            ],
            "model": "claude-3-sonnet-20240229",
            "stop_reason": "end_turn"
        }"#)
}

/// Create a mock for Ollama API
pub fn mock_ollama_api() -> Mock {
    let mut server = Server::new();
    server.mock("POST", "/api/generate")
        .with_header("content-type", "application/json")
        .with_body(r#"{
            "model": "llama2",
            "response": "This is a mocked response from the Ollama API."
        }"#)
}

/// Create a temporary config directory with provider configuration
pub fn setup_temp_config(provider: &str) -> TempDir {
    let temp_dir = tempdir().unwrap();
    let config_dir = temp_dir.path().join(".ola");
    fs::create_dir_all(&config_dir).unwrap();
    
    // Create config file based on provider
    let config_file = config_dir.join("config.yaml");
    let mut server = Server::new();
    let server_url = server.url();
    let config_content = match provider {
        "OpenAI" => format!(r#"
active_provider: "OpenAI"
providers:
  - provider: "OpenAI"
    api_key: "test_key"
    model: "gpt-4"
    additional_settings:
      base_url: "{}"
"#, server_url),
        "Anthropic" => format!(r#"
active_provider: "Anthropic"
providers:
  - provider: "Anthropic"
    api_key: "test_key"
    model: "claude-3-sonnet-20240229"
    additional_settings:
      base_url: "{}"
"#, server_url),
        "Ollama" => format!(r#"
active_provider: "Ollama"
providers:
  - provider: "Ollama"
    api_key: ""
    model: "llama2"
    additional_settings:
      base_url: "{}"
"#, server_url),
        _ => panic!("Unsupported provider: {}", provider),
    };
    
    let mut file = File::create(&config_file).unwrap();
    file.write_all(config_content.as_bytes()).unwrap();
    
    temp_dir
}

/// Create a temporary settings file
pub fn setup_temp_settings() -> TempDir {
    let temp_dir = tempdir().unwrap();
    let settings_dir = temp_dir.path().join(".ola");
    fs::create_dir_all(&settings_dir).unwrap();
    
    let settings_file = settings_dir.join("settings.yaml");
    let settings_content = r#"
default_model: "test_model"
defaults:
  goals: ""
  return_format: "text"
  warnings: ""
  quiet: false
  clipboard: false
behavior:
  enable_logging: false
  log_file: "ola.log"
"#;
    
    let mut file = File::create(&settings_file).unwrap();
    file.write_all(settings_content.as_bytes()).unwrap();
    
    temp_dir
}