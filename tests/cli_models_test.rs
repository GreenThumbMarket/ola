use assert_cmd::Command;
use predicates::prelude::*;
use std::fs::{self, File};
use std::io::Write;
use tempfile::{tempdir, TempDir};

// Helper function to create a temporary config file with a provider
fn setup_temp_config_dir() -> TempDir {
    let temp_dir = tempdir().unwrap();
    let config_dir = temp_dir.path().join(".ola");
    fs::create_dir_all(&config_dir).unwrap();
    
    // Create a config file with a test provider
    let config_file = config_dir.join("config.yaml");
    let config_content = r#"
active_provider: "OpenAI"
providers:
  - provider: "OpenAI"
    api_key: "test_key"
    model: "gpt-4"
"#;
    
    let mut file = File::create(&config_file).unwrap();
    file.write_all(config_content.as_bytes()).unwrap();
    
    temp_dir
}

#[test]
fn test_models_help() {
    // Test help text for the models command
    let mut cmd = Command::cargo_bin("ola").unwrap();
    let output = cmd.arg("models").arg("--help").assert().success();
    
    output.stdout(predicate::str::contains("--provider"));
    output.stdout(predicate::str::contains("--quiet"));
}

#[test]
fn test_models_with_provider() {
    // Test listing models with a specific provider
    let temp_dir = setup_temp_config_dir();
    let old_home = std::env::var("HOME").ok();
    
    // Set HOME to our temp directory
    std::env::set_var("HOME", temp_dir.path());
    
    let mut cmd = Command::cargo_bin("ola").unwrap();
    let output = cmd.arg("models")
        .arg("--provider")
        .arg("OpenAI")
        .assert()
        .success();
    
    output.stdout(predicate::str::contains("OpenAI models"));
    output.stdout(predicate::str::contains("gpt-4"));
    
    // Restore the original HOME
    if let Some(home) = old_home {
        std::env::set_var("HOME", home);
    }
}

#[test]
fn test_models_quiet_mode() {
    // Test listing models in quiet mode
    let temp_dir = setup_temp_config_dir();
    let old_home = std::env::var("HOME").ok();
    
    // Set HOME to our temp directory
    std::env::set_var("HOME", temp_dir.path());
    
    let mut cmd = Command::cargo_bin("ola").unwrap();
    let output = cmd.arg("models")
        .arg("--provider")
        .arg("OpenAI")
        .arg("--quiet")
        .assert()
        .success();
    
    // In quiet mode, we should just see the model names without headers
    output.stdout(predicate::str::contains("gpt-4"));
    output.stdout(predicate::str::contains("gpt-3.5-turbo"));
    
    // Headers should not be present
    output.stdout(predicate::str::contains("OpenAI models").not());
    
    // Restore the original HOME
    if let Some(home) = old_home {
        std::env::set_var("HOME", home);
    }
}

#[test]
fn test_models_with_default_provider() {
    // Test listing models using the default provider from config
    let temp_dir = setup_temp_config_dir();
    let old_home = std::env::var("HOME").ok();
    
    // Set HOME to our temp directory
    std::env::set_var("HOME", temp_dir.path());
    
    let mut cmd = Command::cargo_bin("ola").unwrap();
    let output = cmd.arg("models")
        .assert()
        .success();
    
    // Since OpenAI is the active provider in our test config,
    // we should see OpenAI models
    output.stdout(predicate::str::contains("OpenAI models"));
    
    // Restore the original HOME
    if let Some(home) = old_home {
        std::env::set_var("HOME", home);
    }
}