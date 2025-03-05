use dialoguer::{theme::ColorfulTheme, Input, Password, Select};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProviderConfig {
    pub provider: String,
    pub api_key: String,
    pub model: Option<String>,
    pub additional_settings: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    #[serde(default)]
    pub active_provider: String,
    #[serde(default)]
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
        
        // Depending on file extension, use either JSON or YAML
        let config = if config_path.extension().and_then(|e| e.to_str()) == Some("json") {
            serde_json::from_str(&config_str)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
        } else if config_path.extension().and_then(|e| e.to_str()) == Some("yaml") || 
                  config_path.extension().and_then(|e| e.to_str()) == Some("yml") {
            serde_yaml::from_str(&config_str)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
        } else {
            // Default to JSON for backward compatibility
            serde_json::from_str(&config_str)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
        };
        
        Ok(config)
    }

    pub fn save(&self) -> Result<(), io::Error> {
        let config_path = get_config_path()?;
        let config_dir = config_path.parent().unwrap();
        fs::create_dir_all(config_dir)?;

        // Serialize based on file extension
        let config_str = if config_path.extension().and_then(|e| e.to_str()) == Some("json") {
            serde_json::to_string_pretty(self)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
        } else if config_path.extension().and_then(|e| e.to_str()) == Some("yaml") || 
                  config_path.extension().and_then(|e| e.to_str()) == Some("yml") {
            serde_yaml::to_string(self)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
        } else {
            // Default to JSON for backward compatibility
            serde_json::to_string_pretty(self)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
        };
        
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
        if let Some(existing) = self
            .providers
            .iter_mut()
            .find(|p| p.provider == provider_name)
        {
            *existing = provider;
        } else {
            self.providers.push(provider);
        }
        self.active_provider = provider_name;
    }

    pub fn get_active_provider(&self) -> Option<&ProviderConfig> {
        self.providers
            .iter()
            .find(|p| p.provider == self.active_provider)
    }
}

fn get_config_path() -> Result<PathBuf, io::Error> {
    let home = std::env::var("HOME")
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "HOME directory not found"))?;
    
    // Check for settings.yaml first
    let yaml_path = PathBuf::from(home.clone()).join(".ola").join("settings.yaml");
    if yaml_path.exists() {
        return Ok(yaml_path);
    }
    
    // Backward compatibility: use config.json if it exists
    let json_path = PathBuf::from(home).join(".ola").join("config.json");
    if json_path.exists() {
        return Ok(json_path);
    }
    
    // Default to YAML for new installs
    Ok(yaml_path)
}

// This function is now replaced by the implementation in main.rs
// Keeping the code as a reference, but marking it as private
fn _run_interactive_config() -> Result<(), io::Error> {
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

pub fn fetch_ollama_models() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;
    
    let response = client
        .get("http://localhost:11434/api/tags")
        .send()?;
    
    if !response.status().is_success() {
        return Err(format!("Ollama API error: {}", response.status()).into());
    }
    
    let models_response: serde_json::Value = response.json()?;
    let mut model_names = Vec::new();
    
    if let Some(models) = models_response["models"].as_array() {
        for model in models {
            if let Some(name) = model["name"].as_str() {
                model_names.push(name.to_string());
            }
        }
    }
    
    Ok(model_names)
}

pub fn validate_provider_config(config: &ProviderConfig) -> Result<(), String> {
    // Provider-specific validation
    match config.provider.as_str() {
        "OpenAI" => {
            // Validate API key format and presence
            if config.api_key.trim().is_empty() {
                return Err("API key cannot be empty".to_string());
            }

            if !config.api_key.starts_with("sk-") {
                return Err("OpenAI API key should start with 'sk-'".to_string());
            }

            if config.model.is_none() {
                return Err("OpenAI requires a model name".to_string());
            }
        }
        "Anthropic" => {
            // Validate API key format and presence
            if config.api_key.trim().is_empty() {
                return Err("API key cannot be empty".to_string());
            }

            // Anthropic keys typically start with 'sk-ant-'
            if !config.api_key.starts_with("sk-ant-") {
                return Err("Anthropic API key should start with 'sk-ant-'".to_string());
            }

            if config.model.is_none() {
                return Err("Anthropic requires a model name".to_string());
            }
        }
        "Ollama" => {
            // For Ollama, API key can be empty (local service)

            // Validate Ollama configuration
            if config.model.is_none() {
                return Err("Ollama requires a model name".to_string());
            }
        }
        _ => return Err(format!("Unsupported provider: {}", config.provider)),
    }

    Ok(())
}

// Module-level functions for use in main.rs
pub fn add_provider(provider: ProviderConfig) {
    match Config::load() {
        Ok(mut config) => {
            config.add_provider(provider);
            if let Err(e) = config.save() {
                eprintln!("Failed to save config: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Failed to load config: {}", e);
        }
    }
}

pub fn save() -> Result<(), std::io::Error> {
    match Config::load() {
        Ok(config) => config.save(),
        Err(e) => Err(e),
    }
}
