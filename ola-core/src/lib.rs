// Ola Core Library
// Shared functionality for both CLI and Web interfaces

pub mod api;
pub mod config;
pub mod models;
pub mod project;
pub mod prompt;
pub mod settings;
pub mod utils;

// Re-export commonly used types
pub use config::Config;
pub use models::{Project, Goal, Context, ProjectFile};
pub use settings::Settings;
