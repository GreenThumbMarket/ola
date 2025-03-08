use std::fs::{self, File};
use std::io::Write;
use mockito::{Server, Mock};
use tempfile::{tempdir, TempDir};
use std::env;
use ola::prompt::{self, PromptResult};

// Create a mock for OpenAI API
fn mock_openai_api() -> Mock {
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

// Create a mock for Anthropic API
fn mock_anthropic_api() -> Mock {
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

// Create a temporary config directory with provider configuration
fn setup_temp_config(provider: &str) -> TempDir {
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

#[test]
#[ignore]
fn test_structure_reasoning_openai() {
    // Set up the mock server
    let _m = mock_openai_api();
    
    // Set up a temporary config with OpenAI and the mock URL
    let temp_dir = setup_temp_config("OpenAI");
    let old_home = env::var("HOME").ok();
    env::set_var("HOME", temp_dir.path());
    
    // Call the function under test
    let result = prompt::structure_reasoning_test(
        "Test prompt",
        "text",
        "",
        false,
        None,
        false
    );
    
    // Check that we got a successful result with the expected content
    assert!(result.is_ok());
    if let Ok(PromptResult { content, .. }) = result {
        assert!(content.contains("mocked response from the OpenAI API"));
    }
    
    // Clean up
    if let Some(home) = old_home {
        env::set_var("HOME", home);
    }
}

#[test]
#[ignore]
fn test_structure_reasoning_anthropic() {
    // Set up the mock server
    let _m = mock_anthropic_api();
    
    // Set up a temporary config with Anthropic and the mock URL
    let temp_dir = setup_temp_config("Anthropic");
    let old_home = env::var("HOME").ok();
    env::set_var("HOME", temp_dir.path());
    
    // Call the function under test
    let result = prompt::structure_reasoning_test(
        "Test prompt",
        "text",
        "",
        false,
        None,
        false
    );
    
    // Check that we got a successful result with the expected content
    assert!(result.is_ok());
    if let Ok(PromptResult { content, .. }) = result {
        assert!(content.contains("mocked response from the Anthropic API"));
    }
    
    // Clean up
    if let Some(home) = old_home {
        env::set_var("HOME", home);
    }
}

#[test]
#[ignore]
fn test_stream_non_think_openai() {
    // Set up the mock server
    let _m = mock_openai_api();
    
    // Set up a temporary config with OpenAI and the mock URL
    let temp_dir = setup_temp_config("OpenAI");
    let old_home = env::var("HOME").ok();
    env::set_var("HOME", temp_dir.path());
    
    // Call the function under test
    let result = prompt::stream_non_think_test(
        "Test prompt",
        false,
        None,
        false
    );
    
    // Check that we got a successful result with the expected content
    assert!(result.is_ok());
    if let Ok(PromptResult { content, .. }) = result {
        assert!(content.contains("mocked response from the OpenAI API"));
    }
    
    // Clean up
    if let Some(home) = old_home {
        env::set_var("HOME", home);
    }
}

#[test]
#[ignore]
fn test_with_context() {
    // Set up the mock server
    let _m = mock_openai_api();
    
    // Set up a temporary config with OpenAI and the mock URL
    let temp_dir = setup_temp_config("OpenAI");
    let old_home = env::var("HOME").ok();
    env::set_var("HOME", temp_dir.path());
    
    // Call the function under test with context
    let result = prompt::structure_reasoning_test(
        "Test prompt",
        "text",
        "",
        false,
        Some("This is context from stdin"),
        false
    );
    
    // Check that we got a successful result
    assert!(result.is_ok());
    
    // Clean up
    if let Some(home) = old_home {
        env::set_var("HOME", home);
    }
}

#[test]
#[ignore]
fn test_with_clipboard() {
    // Set up the mock server
    let _m = mock_openai_api();
    
    // Set up a temporary config with OpenAI and the mock URL
    let temp_dir = setup_temp_config("OpenAI");
    let old_home = env::var("HOME").ok();
    env::set_var("HOME", temp_dir.path());
    
    // Call the function under test with clipboard option
    // Note: This won't actually copy to clipboard in tests,
    // but it should exercise the clipboard code path
    let result = prompt::structure_reasoning_test(
        "Test prompt",
        "text",
        "",
        true,  // enable clipboard
        None,
        false
    );
    
    // Check that we got a successful result
    assert!(result.is_ok());
    
    // Clean up
    if let Some(home) = old_home {
        env::set_var("HOME", home);
    }
}

#[test]
#[ignore]
fn test_with_thinking_hidden() {
    // Set up the mock server
    let _m = mock_openai_api();
    
    // Set up a temporary config with OpenAI and the mock URL
    let temp_dir = setup_temp_config("OpenAI");
    let old_home = env::var("HOME").ok();
    env::set_var("HOME", temp_dir.path());
    
    // Call the function under test with thinking hidden
    let result = prompt::structure_reasoning_test(
        "Test prompt",
        "text",
        "",
        false,
        None,
        true  // hide thinking
    );
    
    // Check that we got a successful result
    assert!(result.is_ok());
    
    // Clean up
    if let Some(home) = old_home {
        env::set_var("HOME", home);
    }
}