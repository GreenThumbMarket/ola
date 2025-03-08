mod common;

use mockito;
use std::env;
use ola::prompt::{self, PromptResult};

#[test]
fn test_structure_reasoning_openai() {
    // Set up the mock server
    let _m = common::mock_openai_api();
    
    // Set up a temporary config with OpenAI and the mock URL
    let temp_dir = common::setup_temp_config("OpenAI");
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
fn test_structure_reasoning_anthropic() {
    // Set up the mock server
    let _m = common::mock_anthropic_api();
    
    // Set up a temporary config with Anthropic and the mock URL
    let temp_dir = common::setup_temp_config("Anthropic");
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
fn test_stream_non_think_openai() {
    // Set up the mock server
    let _m = common::mock_openai_api();
    
    // Set up a temporary config with OpenAI and the mock URL
    let temp_dir = common::setup_temp_config("OpenAI");
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
fn test_with_context() {
    // Set up the mock server
    let _m = common::mock_openai_api();
    
    // Set up a temporary config with OpenAI and the mock URL
    let temp_dir = common::setup_temp_config("OpenAI");
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
fn test_with_clipboard() {
    // Set up the mock server
    let _m = common::mock_openai_api();
    
    // Set up a temporary config with OpenAI and the mock URL
    let temp_dir = common::setup_temp_config("OpenAI");
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
fn test_with_thinking_hidden() {
    // Set up the mock server
    let _m = common::mock_openai_api();
    
    // Set up a temporary config with OpenAI and the mock URL
    let temp_dir = common::setup_temp_config("OpenAI");
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