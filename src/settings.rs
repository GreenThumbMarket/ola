use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;

/// Application settings structure for customizing ola behavior
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Settings {
    /// Default model to use when not specified
    #[serde(default = "default_model")]
    pub default_model: String,
    
    /// Default prompt template customization
    #[serde(default)]
    pub prompt_template: PromptTemplate,
    
    /// Defaults for command flags
    #[serde(default)]
    pub defaults: DefaultSettings,
    
    /// Behavior customization settings
    #[serde(default)]
    pub behavior: BehaviorSettings,
}

/// Settings for the prompt template
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PromptTemplate {
    /// Text to display before the goals section
    #[serde(default = "default_goals_prefix")]
    pub goals_prefix: String,
    
    /// Text to display before the return format section
    #[serde(default = "default_return_format_prefix")]
    pub return_format_prefix: String,
    
    /// Text to display before the warnings section
    #[serde(default = "default_warnings_prefix")]
    pub warnings_prefix: String,
}

/// Default settings for command flags
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DefaultSettings {
    /// Default return format when not specified
    #[serde(default = "default_return_format")]
    pub return_format: String,
    
    /// Default to quiet mode
    #[serde(default)]
    pub quiet: bool,
    
    /// Default to no-thinking mode
    #[serde(default)]
    pub no_thinking: bool,
    
    /// Default to copying results to clipboard
    #[serde(default)]
    pub clipboard: bool,
}

/// Behavior customization settings
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct BehaviorSettings {
    /// Log file location for session outputs
    #[serde(default = "default_log_file")]
    pub log_file: String,
    
    /// Enable or disable session logging
    #[serde(default = "default_enable_logging")]
    pub enable_logging: bool,
    
    /// Thinking animation customization
    #[serde(default)]
    pub thinking_animation: ThinkingAnimation,
}

/// Settings for thinking animation
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ThinkingAnimation {
    /// Emoji sequence to use for the thinking animation
    #[serde(default = "default_thinking_emojis")]
    pub emojis: Vec<String>,
    
    /// Text to display with the thinking animation
    #[serde(default = "default_thinking_text")]
    pub text: String,
}

// Default implementations
fn default_model() -> String {
    "gpt-5".to_string()
}

fn default_goals_prefix() -> String {
    "🏆 Goals: ".to_string()
}

fn default_return_format_prefix() -> String {
    "📝 Return Format: ".to_string()
}

fn default_warnings_prefix() -> String {
    "⚠️ Warnings: ".to_string()
}

fn default_return_format() -> String {
    "text".to_string()
}

fn default_log_file() -> String {
    "sessions.jsonl".to_string()
}

fn default_enable_logging() -> bool {
    true
}

fn default_thinking_emojis() -> Vec<String> {
    vec!["🌊".to_string(), "🏄".to_string(), "🌊".to_string(), "🏄‍♀️".to_string()]
}

fn default_thinking_text() -> String {
    "thinking...".to_string()
}

impl Default for ThinkingAnimation {
    fn default() -> Self {
        Self {
            emojis: default_thinking_emojis(),
            text: default_thinking_text(),
        }
    }
}

impl Settings {
    /// Load settings from file, or create default settings if the file doesn't exist
    pub fn load() -> Result<Self, io::Error> {
        let settings_path = get_settings_path()?;
        if !settings_path.exists() {
            let settings = Settings::default();
            settings.save()?;
            return Ok(settings);
        }

        let settings_str = fs::read_to_string(&settings_path)?;
        let settings = serde_yaml::from_str(&settings_str)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(settings)
    }

    /// Save settings to file
    pub fn save(&self) -> Result<(), io::Error> {
        let settings_path = get_settings_path()?;
        let settings_dir = settings_path.parent().unwrap();
        fs::create_dir_all(settings_dir)?;

        let settings_str = serde_yaml::to_string(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(&settings_path, settings_str)?;

        Ok(())
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            default_model: default_model(),
            prompt_template: PromptTemplate::default(),
            defaults: DefaultSettings::default(),
            behavior: BehaviorSettings::default(),
        }
    }
}

/// Get the path to the settings file
fn get_settings_path() -> Result<PathBuf, io::Error> {
    let home = std::env::var("HOME")
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "HOME directory not found"))?;
    
    Ok(PathBuf::from(home).join(".ola").join("settings.yaml"))
}