use assert_cmd::Command;
use predicates::prelude::*;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use tempfile::{tempdir, TempDir};

// Helper function to create a temporary settings file
fn setup_temp_settings_dir() -> TempDir {
    let temp_dir = tempdir().unwrap();
    let settings_dir = temp_dir.path().join(".ola");
    fs::create_dir_all(&settings_dir).unwrap();
    
    // Create a minimal settings file
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

#[test]
#[ignore]
fn test_settings_help() {
    // Test help text for the settings command
    let mut cmd = Command::cargo_bin("ola").unwrap();
    let output = cmd.arg("settings").arg("--help").output().expect("Failed to execute command");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--view"));
    assert!(stdout.contains("--default-model"));
    assert!(stdout.contains("--default-format"));
    assert!(stdout.contains("--logging"));
    assert!(stdout.contains("--log-file"));
    assert!(stdout.contains("--reset"));
}

#[test]
fn test_settings_view() {
    // Test viewing settings
    let temp_dir = setup_temp_settings_dir();
    let old_home = std::env::var("HOME").ok();
    
    // Set HOME to our temp directory to redirect config file loading
    std::env::set_var("HOME", temp_dir.path());
    
    let mut cmd = Command::cargo_bin("ola").unwrap();
    let output = cmd.arg("settings")
        .arg("--view")
        .output()
        .expect("Failed to execute command");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Current settings"));
    assert!(stdout.contains("default_model"));
    assert!(stdout.contains("test_model"));
    
    // Restore the original HOME
    if let Some(home) = old_home {
        std::env::set_var("HOME", home);
    }
}

#[test]
#[ignore]
fn test_settings_update_default_model() {
    // Test updating the default model
    let temp_dir = setup_temp_settings_dir();
    let old_home = std::env::var("HOME").ok();
    
    // Set HOME to our temp directory
    std::env::set_var("HOME", temp_dir.path());
    
    let mut cmd = Command::cargo_bin("ola").unwrap();
    let output = cmd.arg("settings")
        .arg("--default-model")
        .arg("new_model")
        .output()
        .expect("Failed to execute command");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Default model set to: new_model"));
    
    // Verify the settings file was updated
    let settings_file = temp_dir.path().join(".ola").join("settings.yaml");
    let settings_content = fs::read_to_string(settings_file).unwrap();
    assert!(settings_content.contains("default_model: new_model"));
    
    // Restore the original HOME
    if let Some(home) = old_home {
        std::env::set_var("HOME", home);
    }
}

#[test]
#[ignore]
fn test_settings_update_default_format() {
    // Test updating the default format
    let temp_dir = setup_temp_settings_dir();
    let old_home = std::env::var("HOME").ok();
    
    // Set HOME to our temp directory
    std::env::set_var("HOME", temp_dir.path());
    
    let mut cmd = Command::cargo_bin("ola").unwrap();
    let output = cmd.arg("settings")
        .arg("--default-format")
        .arg("json")
        .output()
        .expect("Failed to execute command");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Default return format set to: json"));
    
    // Restore the original HOME
    if let Some(home) = old_home {
        std::env::set_var("HOME", home);
    }
}

#[test]
fn test_settings_reset() {
    // Test resetting settings
    let temp_dir = setup_temp_settings_dir();
    let old_home = std::env::var("HOME").ok();
    
    // Set HOME to our temp directory
    std::env::set_var("HOME", temp_dir.path());
    
    let mut cmd = Command::cargo_bin("ola").unwrap();
    let output = cmd.arg("settings")
        .arg("--reset")
        .output()
        .expect("Failed to execute command");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Settings reset to default values"));
    
    // Restore the original HOME
    if let Some(home) = old_home {
        std::env::set_var("HOME", home);
    }
}