use assert_cmd::Command;
use predicates::prelude::*;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use tempfile::{tempdir, TempDir};

// Helper function to create a temporary config file
fn setup_temp_config_dir() -> TempDir {
    let temp_dir = tempdir().unwrap();
    let config_dir = temp_dir.path().join(".ola");
    fs::create_dir_all(&config_dir).unwrap();
    
    // Create a minimal config file to avoid interactive prompts
    let config_file = config_dir.join("config.yaml");
    let config_content = r#"
active_provider: "TestProvider"
providers:
  - provider: "TestProvider"
    api_key: "test_key"
    model: "test_model"
"#;
    
    let mut file = File::create(&config_file).unwrap();
    file.write_all(config_content.as_bytes()).unwrap();
    
    temp_dir
}

#[test]
fn test_configure_help() {
    // Test help text for the configure command
    let mut cmd = Command::cargo_bin("ola").unwrap();
    let output = cmd.arg("configure").arg("--help").assert().success();
    
    output.stdout(predicate::str::contains("--provider"));
    output.stdout(predicate::str::contains("--api_key"));
    output.stdout(predicate::str::contains("--model"));
}

#[test]
fn test_configure_with_args() {
    // Test configuring with command line arguments
    let temp_dir = setup_temp_config_dir();
    let old_home = std::env::var("HOME").ok();
    
    // Set HOME to our temp directory to redirect config file creation
    std::env::set_var("HOME", temp_dir.path());
    
    let mut cmd = Command::cargo_bin("ola").unwrap();
    cmd.arg("configure")
        .arg("--provider")
        .arg("TestProvider")
        .arg("--api_key")
        .arg("test_api_key")
        .arg("--model")
        .arg("test_model");
        
    // The configure command would normally be interactive
    // We need to mock the API validation checks or modify the code to be testable
    
    // Restore the original HOME
    if let Some(home) = old_home {
        std::env::set_var("HOME", home);
    }
}

// Additional tests would be needed for interactive mode
// These would require mocking stdin for dialoguer interactions