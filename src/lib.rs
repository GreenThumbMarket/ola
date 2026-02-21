// This library crate exposes the core functionality of ola
// for unit testing and reuse

// Core modules
pub mod config;
pub mod models;
pub mod project;
pub mod prompt;
pub mod settings;

// API communication layer
pub mod api;

// Utility modules
pub mod utils;

// Re-export the main components
pub use api::ApiClient;
pub use config::{Config, ProviderConfig};
pub use models::{Context, Goal, Project, ProjectFile};
pub use project::ProjectManager;
pub use settings::Settings;
