#[macro_use]
extern crate rocket;

use rocket::State;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::fs::FileServer;
use rocket_dyn_templates::{Template, context};
use ola_core::{Config, api, config::ProviderConfig};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::env;

// Shared application state
struct AppState {
    config: Arc<Mutex<Option<Config>>>,
}

// Helper function to get API key from environment variables
fn get_api_key_from_env(provider: &str) -> Result<String, String> {
    let env_var = match provider.to_lowercase().as_str() {
        "openai" => "OPENAI_API_KEY",
        "anthropic" => "ANTHROPIC_API_KEY",
        "gemini" => "GEMINI_API_KEY",
        "ollama" => return Ok(String::new()), // Ollama doesn't need an API key
        _ => return Err(format!("Unknown provider: {}", provider)),
    };

    env::var(env_var)
        .map_err(|_| format!("Environment variable {} not found. Please set it in your .env file.", env_var))
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct PromptRequest {
    goal: String,
    format: String,
    warnings: String,
    context: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct PromptResponse {
    success: bool,
    response: Option<String>,
    error: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct ConfigRequest {
    provider: String,
    model: String,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct ConfigResponse {
    success: bool,
    message: String,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct TestConnectionRequest {
    provider: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct TestConnectionResponse {
    success: bool,
    message: String,
    details: Option<ConnectionDetails>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct ConnectionDetails {
    provider: String,
    model: String,
    api_key_valid: bool,
    endpoint_responding: bool,
}

#[get("/")]
fn index() -> Template {
    Template::render("index", context! {
        title: "Ola - AI Assistant"
    })
}

#[get("/configure")]
fn configure_page() -> Template {
    Template::render("configure", context! {
        title: "Configure Ola"
    })
}

#[post("/api/configure", data = "<config_req>")]
async fn api_configure(
    config_req: Json<ConfigRequest>,
    state: &State<AppState>,
) -> Json<ConfigResponse> {
    // Get API key from environment variables
    let api_key = match get_api_key_from_env(&config_req.provider) {
        Ok(key) => key,
        Err(e) => {
            return Json(ConfigResponse {
                success: false,
                message: e,
            });
        }
    };

    let provider_config = ProviderConfig {
        provider: config_req.provider.clone(),
        api_key,
        model: Some(config_req.model.clone()),
        additional_settings: None,
    };

    // Validate the configuration
    if let Err(e) = ola_core::config::validate_provider_config(&provider_config) {
        return Json(ConfigResponse {
            success: false,
            message: format!("Configuration validation failed: {}", e),
        });
    }

    // Load or create config
    let mut config = Config::load().unwrap_or_else(|_| Config {
        active_provider: String::new(),
        providers: Vec::new(),
    });

    config.add_provider(provider_config);

    // Save configuration
    match config.save() {
        Ok(_) => {
            let mut state_config = state.config.lock().await;
            *state_config = Some(config);
            Json(ConfigResponse {
                success: true,
                message: "Configuration saved successfully".to_string(),
            })
        }
        Err(e) => Json(ConfigResponse {
            success: false,
            message: format!("Failed to save configuration: {}", e),
        }),
    }
}

#[post("/api/prompt", data = "<prompt_req>")]
async fn api_prompt(
    prompt_req: Json<PromptRequest>,
) -> Json<PromptResponse> {
    // Load fresh config
    let config = match Config::load() {
        Ok(cfg) => cfg,
        Err(_) => {
            return Json(PromptResponse {
                success: false,
                response: None,
                error: Some("No configuration found. Please configure ola first.".to_string()),
            });
        }
    };

    let provider_config = match config.get_active_provider() {
        Some(cfg) => cfg,
        None => {
            return Json(PromptResponse {
                success: false,
                response: None,
                error: Some("No active provider found. Please configure ola first.".to_string()),
            });
        }
    };

    // Get base URL if available
    let base_url = provider_config.additional_settings.as_ref()
        .and_then(|settings| settings.get("base_url"))
        .and_then(|url| url.as_str());

    // Build the full prompt using the Goals/Format/Warnings structure
    let formatted_prompt = api::format_prompt(
        &prompt_req.goal,
        &prompt_req.format,
        &prompt_req.warnings,
        prompt_req.context.as_deref(),
    );

    // Get the model
    let model = provider_config.model.as_deref().unwrap_or("gpt-5");

    // Prepare data for spawn_blocking (all must be Send)
    let provider_name = provider_config.provider.clone();
    let api_key = provider_config.api_key.clone();
    let base_url_opt = base_url.map(|s| s.to_string());
    let formatted_prompt_clone = formatted_prompt.clone();
    let model_clone = model.to_string();

    // Execute the prompt (blocking operation in a separate task)
    let result = tokio::task::spawn_blocking(move || -> Result<String, String> {
        // Reconstruct API client in the blocking task
        let client = api::ApiClient::new(&provider_name, &api_key, base_url_opt.as_deref())
            .map_err(|e| e.to_string())?;
        client.stream_prompt(&formatted_prompt_clone, &model_clone)
            .map_err(|e| e.to_string())
    }).await;

    match result {
        Ok(Ok(response)) => Json(PromptResponse {
            success: true,
            response: Some(response),
            error: None,
        }),
        Ok(Err(e)) => Json(PromptResponse {
            success: false,
            response: None,
            error: Some(format!("Error: {}", e)),
        }),
        Err(e) => Json(PromptResponse {
            success: false,
            response: None,
            error: Some(format!("Task error: {}", e)),
        }),
    }
}

#[post("/api/test-connection", data = "<test_req>")]
async fn api_test_connection(
    test_req: Json<TestConnectionRequest>,
) -> Json<TestConnectionResponse> {
    // Load configuration
    let config = match Config::load() {
        Ok(cfg) => cfg,
        Err(_) => {
            return Json(TestConnectionResponse {
                success: false,
                message: "No configuration found. Please configure a provider first.".to_string(),
                details: None,
            });
        }
    };

    // Determine which provider to test
    let provider_config = if let Some(ref provider_name) = test_req.provider {
        // Test specific provider
        let matching_provider = config.providers.iter()
            .find(|p| p.provider.eq_ignore_ascii_case(provider_name));

        match matching_provider {
            Some(p) => {
                let mut cfg = p.clone();
                // Apply environment variable fallback for API key
                cfg.api_key = match cfg.provider.as_str() {
                    "OpenAI" => std::env::var("OPENAI_API_KEY").unwrap_or(cfg.api_key),
                    "Anthropic" => std::env::var("ANTHROPIC_API_KEY").unwrap_or(cfg.api_key),
                    "Gemini" => std::env::var("GEMINI_API_KEY").unwrap_or(cfg.api_key),
                    _ => cfg.api_key,
                };
                Some(cfg)
            }
            None => {
                return Json(TestConnectionResponse {
                    success: false,
                    message: format!("Provider '{}' not found in configuration.", provider_name),
                    details: None,
                });
            }
        }
    } else {
        // Test active provider
        match config.get_active_provider() {
            Some(cfg) => Some(cfg),
            None => {
                return Json(TestConnectionResponse {
                    success: false,
                    message: "No active provider configured. Please configure a provider first.".to_string(),
                    details: None,
                });
            }
        }
    };

    let provider_config = provider_config.unwrap();
    let provider_name = provider_config.provider.clone();
    let model_name = provider_config.model.clone().unwrap_or_else(|| "default".to_string());

    // Test the connection in a blocking task
    let result = tokio::task::spawn_blocking(move || {
        ola_core::config::test_provider_connection(&provider_config)
            .map_err(|e| e.to_string())
    }).await;

    match result {
        Ok(Ok(_)) => {
            Json(TestConnectionResponse {
                success: true,
                message: format!("Successfully connected to {} with model {}", provider_name, model_name),
                details: Some(ConnectionDetails {
                    provider: provider_name,
                    model: model_name,
                    api_key_valid: true,
                    endpoint_responding: true,
                }),
            })
        }
        Ok(Err(e)) => {
            Json(TestConnectionResponse {
                success: false,
                message: format!("Connection test failed: {}", e),
                details: Some(ConnectionDetails {
                    provider: provider_name,
                    model: model_name,
                    api_key_valid: false,
                    endpoint_responding: false,
                }),
            })
        }
        Err(e) => {
            Json(TestConnectionResponse {
                success: false,
                message: format!("Task error: {}", e),
                details: None,
            })
        }
    }
}

#[get("/api/models")]
async fn api_models(_state: &State<AppState>) -> Json<Vec<String>> {
    let config = Config::load().ok();
    let models = match config.and_then(|c| c.get_active_provider()) {
        Some(provider_config) => match provider_config.provider.as_str() {
            "OpenAI" => vec![
                "gpt-5.1".to_string(),
                "gpt-5".to_string(),
                "gpt-5-mini".to_string(),
                "gpt-5-nano".to_string(),
                "gpt-5-codex".to_string(),
                "gpt-4.1".to_string(),
                "gpt-4.1-mini".to_string(),
                "gpt-4.1-nano".to_string(),
                "o3".to_string(),
                "o3-pro".to_string(),
                "o4-mini".to_string(),
            ],
            "Anthropic" => vec![
                "claude-opus-4-5-20251124".to_string(),
                "claude-haiku-4-5-20251015".to_string(),
                "claude-opus-4-1-20250805".to_string(),
                "claude-sonnet-4-20250522".to_string(),
                "claude-3-opus-20240229".to_string(),
                "claude-3-sonnet-20240229".to_string(),
                "claude-3-haiku-20240307".to_string(),
            ],
            "Gemini" => vec![
                "gemini-3-pro".to_string(),
                "gemini-2.5-pro".to_string(),
                "gemini-2.5-flash".to_string(),
                "gemini-2.5-flash-lite".to_string(),
                "gemini-2.5-flash-image".to_string(),
                "gemini-2.0-flash".to_string(),
                "gemini-1.5-pro".to_string(),
                "gemini-1.5-flash".to_string(),
            ],
            "Ollama" => {
                // Try to fetch Ollama models
                ola_core::config::fetch_ollama_models().unwrap_or_else(|_| vec![
                    "llama2".to_string(),
                    "mistral".to_string(),
                    "codellama".to_string(),
                    "phi".to_string(),
                ])
            },
            _ => vec![],
        },
        None => vec![],
    };
    Json(models)
}

#[launch]
fn rocket() -> _ {
    // Load .env file
    dotenvy::dotenv().ok();

    // Initialize logging
    env_logger::init();

    // Load existing configuration if available
    let config = Config::load().ok();

    let state = AppState {
        config: Arc::new(Mutex::new(config)),
    };

    rocket::build()
        .manage(state)
        .mount("/", routes![index, configure_page])
        .mount("/api", routes![api_configure, api_prompt, api_models, api_test_connection])
        .mount("/static", FileServer::from("ola-web/static"))
        .attach(Template::fairing())
}
