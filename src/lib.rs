// This library crate exposes the core functionality of ola
// for unit testing and reuse

// Core modules
pub mod config;
pub mod prompt;
pub mod settings;

// API communication layer
pub mod api;

// Utility modules
pub mod utils;

// Re-export the main components
pub use config::{Config, ProviderConfig};
pub use settings::Settings;
pub use api::ApiClient;