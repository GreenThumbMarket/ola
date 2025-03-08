// API module for handling provider-specific API interactions
use serde_json::json;
use std::time::Duration;

// Import provider-specific modules
mod openai;
mod anthropic;
mod ollama;

// Provider implementations
pub use openai::OpenAI;
pub use anthropic::Anthropic;
pub use ollama::Ollama;

// Trait for API providers
pub trait Provider {
    fn send_prompt(&self, prompt: &str, model: &str, stream: bool) -> Result<String, Box<dyn std::error::Error>>;
    fn get_provider_name(&self) -> &str;
}

// API client for handling communication with LLM providers
pub struct ApiClient {
    provider: Box<dyn Provider>,
}

impl ApiClient {
    // Create a new API client for the specified provider
    pub fn new(provider_name: &str, api_key: &str, base_url: Option<&str>) -> Result<Self, Box<dyn std::error::Error>> {
        let provider: Box<dyn Provider> = match provider_name {
            "OpenAI" => Box::new(OpenAI::new(api_key, base_url)),
            "Anthropic" => Box::new(Anthropic::new(api_key, base_url)),
            "Ollama" => Box::new(Ollama::new(base_url)),
            _ => return Err(format!("Unsupported provider: {}", provider_name).into()),
        };
        
        Ok(Self { provider })
    }
    
    // Send a prompt to the provider and return the response
    pub fn send_prompt(&self, prompt: &str, model: &str) -> Result<String, Box<dyn std::error::Error>> {
        self.provider.send_prompt(prompt, model, false)
    }
    
    // Send a prompt and stream the response
    pub fn stream_prompt(&self, prompt: &str, model: &str) -> Result<String, Box<dyn std::error::Error>> {
        self.provider.send_prompt(prompt, model, true)
    }
    
    // Get the provider name
    pub fn get_provider_name(&self) -> &str {
        self.provider.get_provider_name()
    }
}

// Factory function to create an API client from configuration
pub fn create_api_client_from_config() -> Result<ApiClient, Box<dyn std::error::Error>> {
    // Load configuration
    let config = crate::config::Config::load()?;
    let provider_config = config.get_active_provider().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No active provider configured. Run 'ola configure' first.",
        )
    })?;
    
    // Extract provider information
    let provider_name = &provider_config.provider;
    let api_key = &provider_config.api_key;
    
    // Check for additional settings like base_url
    let base_url = provider_config.additional_settings.as_ref()
        .and_then(|settings| settings.get("base_url"))
        .and_then(|url| url.as_str());
    
    // Create and return the API client
    ApiClient::new(provider_name, api_key, base_url)
}

// Helper function to format a prompt with context
pub fn format_prompt(goals: &str, return_type: &str, warnings: &str, context: Option<&str>) -> String {
    // Try to load settings for custom prefixes
    let settings = crate::settings::Settings::load().unwrap_or_default();
    
    // Use custom prefixes from settings if available
    let goals_prefix = &settings.prompt_template.goals_prefix;
    let return_format_prefix = &settings.prompt_template.return_format_prefix;
    let warnings_prefix = &settings.prompt_template.warnings_prefix;
    
    // Build the input data with optional context
    if let Some(ctx) = context {
        format!(
            "{}{}\n{}{}\n{}{}\nContext: {}",
            goals_prefix, goals, 
            return_format_prefix, return_type, 
            warnings_prefix, warnings, 
            ctx
        )
    } else {
        format!(
            "{}{}\n{}{}\n{}{}",
            goals_prefix, goals, 
            return_format_prefix, return_type, 
            warnings_prefix, warnings
        )
    }
}