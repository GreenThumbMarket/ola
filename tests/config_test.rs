mod common;

use std::env;
use ola::config::{Config, ProviderConfig, validate_provider_config, add_provider, save, fetch_ollama_models};

#[test]
fn test_config_load() {
    // Set up a temporary config
    let temp_dir = common::setup_temp_config("OpenAI");
    let old_home = env::var("HOME").ok();
    env::set_var("HOME", temp_dir.path());
    
    // Load the config
    let result = Config::load();
    assert!(result.is_ok());
    
    let config = result.unwrap();
    assert_eq!(config.active_provider, "OpenAI");
    assert_eq!(config.providers.len(), 1);
    assert_eq!(config.providers[0].provider, "OpenAI");
    
    // Clean up
    if let Some(home) = old_home {
        env::set_var("HOME", home);
    }
}

#[test]
fn test_validate_provider_config() {
    // Test OpenAI validation
    let openai_config = ProviderConfig {
        provider: "OpenAI".to_string(),
        api_key: "test_key".to_string(),
        model: Some("gpt-4".to_string()),
        additional_settings: None,
    };
    
    let result = validate_provider_config(&openai_config);
    assert!(result.is_ok());
    
    // Test Anthropic validation
    let anthropic_config = ProviderConfig {
        provider: "Anthropic".to_string(),
        api_key: "test_key".to_string(),
        model: Some("claude-3-sonnet-20240229".to_string()),
        additional_settings: None,
    };
    
    let result = validate_provider_config(&anthropic_config);
    assert!(result.is_ok());
    
    // Test Ollama validation
    let ollama_config = ProviderConfig {
        provider: "Ollama".to_string(),
        api_key: "".to_string(),
        model: Some("llama2".to_string()),
        additional_settings: None,
    };
    
    let result = validate_provider_config(&ollama_config);
    assert!(result.is_ok());
    
    // Test validation with missing API key
    let invalid_config = ProviderConfig {
        provider: "OpenAI".to_string(),
        api_key: "".to_string(),
        model: Some("gpt-4".to_string()),
        additional_settings: None,
    };
    
    let result = validate_provider_config(&invalid_config);
    assert!(result.is_err());
}

#[test]
fn test_add_provider() {
    // Set up a temporary config directory
    let temp_dir = common::setup_temp_config("OpenAI");
    let old_home = env::var("HOME").ok();
    env::set_var("HOME", temp_dir.path());
    
    // Add a new provider
    let new_provider = ProviderConfig {
        provider: "Anthropic".to_string(),
        api_key: "test_key".to_string(),
        model: Some("claude-3-sonnet-20240229".to_string()),
        additional_settings: None,
    };
    
    // Add the provider and save
    add_provider(new_provider.clone());
    let result = save();
    assert!(result.is_ok());
    
    // Load the config again to verify the new provider was added
    let config = Config::load().unwrap();
    assert_eq!(config.providers.len(), 2);
    
    // Check if the new provider is in the list
    let found = config.providers.iter().any(|p| p.provider == "Anthropic");
    assert!(found, "The new provider was not added to the config");
    
    // Clean up
    if let Some(home) = old_home {
        env::set_var("HOME", home);
    }
}

#[test]
fn test_fetch_ollama_models() {
    // This test requires a running Ollama instance
    // We'll mock the API response using mockito
    
    let mock_server = mockito::Server::new();
    
    // Create a mock for the Ollama models endpoint
    let _m = mock_server.mock("GET", "/api/tags")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"models":[{"name":"llama2"},{"name":"mistral"}]}"#)
        .create();
    
    // Set an environment variable to override the Ollama URL
    env::set_var("OLLAMA_HOST", mock_server.url());
    
    // Call the function
    let result = fetch_ollama_models();
    assert!(result.is_ok());
    
    let models = result.unwrap();
    assert_eq!(models.len(), 2);
    assert!(models.contains(&"llama2".to_string()));
    assert!(models.contains(&"mistral".to_string()));
    
    // Clean up
    env::remove_var("OLLAMA_HOST");
}