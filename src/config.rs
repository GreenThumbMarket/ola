use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use dialoguer::{theme::ColorfulTheme, Input, Select, Password};
use std::io;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProviderConfig {
    pub provider: String,
    pub api_key: String,
    pub model: Option<String>,
    pub additional_settings: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub active_provider: String,
    pub providers: Vec<ProviderConfig>,
}

impl Config {
    pub fn load() -> Result<Self, io::Error> {
        let config_path = get_config_path()?;
        if !config_path.exists() {
            return Ok(Config {
                active_provider: String::new(),
                providers: Vec::new(),
            });
        }

        let config_str = fs::read_to_string(&config_path)?;
        let config = serde_json::from_str(&config_str)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(config)
    }

    pub fn save(&self) -> Result<(), io::Error> {
        let config_path = get_config_path()?;
        let config_dir = config_path.parent().unwrap();
        fs::create_dir_all(config_dir)?;
        
        let config_str = serde_json::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(&config_path, config_str)?;
        
        // Set restrictive permissions on config file (600)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&config_path)?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&config_path, perms)?;
        }
        
        Ok(())
    }

    pub fn add_provider(&mut self, provider: ProviderConfig) {
        let provider_name = provider.provider.clone();
        if let Some(existing) = self.providers.iter_mut()
            .find(|p| p.provider == provider_name) {
            *existing = provider;
        } else {
            self.providers.push(provider);
        }
        self.active_provider = provider_name;
    }

    pub fn get_active_provider(&self) -> Option<&ProviderConfig> {
        self.providers.iter()
            .find(|p| p.provider == self.active_provider)
    }
}

fn get_config_path() -> Result<PathBuf, io::Error> {
    let home = std::env::var("HOME")
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "HOME directory not found"))?;
    Ok(PathBuf::from(home).join(".ola").join("config.json"))
}

pub fn run_interactive_config() -> Result<(), io::Error> {
    let mut config = Config::load()?;
    
    println!("🤖 Welcome to Ola Configuration!");
    
    // Provider selection
    let providers = vec!["OpenAI", "Anthropic", "Ollama"];
    let selected_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select your LLM provider")
        .items(&providers)
        .default(0)
        .interact()
        .unwrap();
    let provider = providers[selected_idx].to_string();
    
    // Get API key securely
    let api_key = Password::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Enter your {} API key", provider))
        .interact()
        .unwrap();

    // Model selection based on provider
    let model = match provider.as_str() {
        "OpenAI" => {
            let models = vec!["gpt-4", "gpt-3.5-turbo"];
            let idx = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select model")
                .items(&models)
                .default(0)
                .interact()
                .unwrap();
            Some(models[idx].to_string())
        }
        "Anthropic" => {
            let models = vec!["claude-2", "claude-instant"];
            let idx = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select model")
                .items(&models)
                .default(0)
                .interact()
                .unwrap();
            Some(models[idx].to_string())
        }
        "Ollama" => {
            let model: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter model name (e.g., llama2, codellama)")
                .interact_text()
                .unwrap();
            Some(model)
        }
        _ => None,
    };

    let provider_config = ProviderConfig {
        provider,
        api_key,
        model,
        additional_settings: None,
    };

    config.add_provider(provider_config);
    config.save()?;

    println!("✅ Configuration saved successfully!");
    Ok(())
}

pub fn validate_provider_config(config: &ProviderConfig) -> Result<(), String> {
    // Validate API key format and presence
    if config.api_key.trim().is_empty() {
        return Err("API key cannot be empty".to_string());
    }

    // Provider-specific validation
    match config.provider.as_str() {
        "OpenAI" => {
            if !config.api_key.starts_with("sk-") {
                return Err("OpenAI API key should start with 'sk-'".to_string());
            }
        }
        "Anthropic" => {
            // Add Anthropic-specific validation if needed
        }
        "Ollama" => {
            // Validate Ollama configuration
            if config.model.is_none() {
                return Err("Ollama requires a model name".to_string());
            }
        }
        _ => return Err(format!("Unsupported provider: {}", config.provider)),
    }

    Ok(())
}