// Main test module for the ola crate
mod common;

// Integration tests for CLI functionality
mod cli_main_test;
mod cli_prompt_test;
mod cli_non_think_test;
mod cli_configure_test;
mod cli_settings_test;
mod cli_models_test; 
mod cli_session_test;

// Unit tests for ola modules
mod config_test;
mod prompt_test;
mod settings_test;

// Each test module covers a specific area of functionality
// Tests are run with `cargo test`