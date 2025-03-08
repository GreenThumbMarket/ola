mod common;

use std::env;
use ola::settings::{Settings, DefaultSettings, BehaviorSettings};

#[test]
fn test_settings_load() {
    // Set up a temporary settings directory
    let temp_dir = common::setup_temp_settings();
    let old_home = env::var("HOME").ok();
    env::set_var("HOME", temp_dir.path());
    
    // Load the settings
    let result = Settings::load();
    assert!(result.is_ok());
    
    let settings = result.unwrap();
    assert_eq!(settings.default_model, "test_model");
    assert_eq!(settings.defaults.return_format, "text");
    assert_eq!(settings.behavior.enable_logging, false);
    
    // Clean up
    if let Some(home) = old_home {
        env::set_var("HOME", home);
    }
}

#[test]
fn test_settings_default() {
    // Create default settings
    let settings = Settings::default();
    
    // Check the default values
    assert_eq!(settings.default_model, "default");
    assert_eq!(settings.defaults.return_format, "text");
    assert_eq!(settings.defaults.quiet, false);
    assert_eq!(settings.defaults.clipboard, false);
    assert_eq!(settings.behavior.enable_logging, false);
    assert_eq!(settings.behavior.log_file, "ola.log");
}

#[test]
fn test_settings_save() {
    // Set up a temporary settings directory
    let temp_dir = common::setup_temp_settings();
    let old_home = env::var("HOME").ok();
    env::set_var("HOME", temp_dir.path());
    
    // Create custom settings
    let settings = Settings {
        default_model: "custom_model".to_string(),
        defaults: DefaultSettings {
            goals: "".to_string(),
            return_format: "json".to_string(),
            warnings: "".to_string(),
            quiet: true,
            clipboard: true,
        },
        behavior: BehaviorSettings {
            enable_logging: true,
            log_file: "custom.log".to_string(),
        },
    };
    
    // Save the settings
    let result = settings.save();
    assert!(result.is_ok());
    
    // Load the settings again to verify changes
    let loaded = Settings::load().unwrap();
    assert_eq!(loaded.default_model, "custom_model");
    assert_eq!(loaded.defaults.return_format, "json");
    assert_eq!(loaded.defaults.quiet, true);
    assert_eq!(loaded.defaults.clipboard, true);
    assert_eq!(loaded.behavior.enable_logging, true);
    assert_eq!(loaded.behavior.log_file, "custom.log");
    
    // Clean up
    if let Some(home) = old_home {
        env::set_var("HOME", home);
    }
}

#[test]
fn test_settings_missing_file() {
    // Set up a temporary directory without a settings file
    let temp_dir = tempfile::tempdir().unwrap();
    let old_home = env::var("HOME").ok();
    env::set_var("HOME", temp_dir.path());
    
    // Loading settings should return an error
    let result = Settings::load();
    assert!(result.is_err());
    
    // But we should be able to create and save new settings
    let settings = Settings::default();
    let save_result = settings.save();
    assert!(save_result.is_ok());
    
    // Now loading should succeed
    let load_result = Settings::load();
    assert!(load_result.is_ok());
    
    // Clean up
    if let Some(home) = old_home {
        env::set_var("HOME", home);
    }
}