/*
.olaHints content:
- This file contains hints for optimizing model calls.
- Use the format [Goals, Return Format, Warnings] when configuring sessions.
- Providers and session storage are easily configurable.
*/

use chrono::Utc;
use clap::Parser;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use serde_json::json;
use std::fs::OpenOptions;
use std::io::Write;

// Core modules
mod config;
mod prompt;
mod settings;

// API communication layer
mod api;

// Utility modules
mod utils;

#[derive(Parser)]
#[command(name = "ola")]
#[command(version = "0.2.0")]
#[command(about = "A friendly CLI for prompting and optimizing reasoning model calls", long_about = None)]
struct OlaCli {
    /// Specify a subcommand
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Starts the application with optional arguments
    Start {
        /// Optional parameter for demonstration
        #[arg(short, long)]
        verbose: bool,
    },

    /// Prompt command with optional flags for goals, format, and warnings
    Prompt {
        /// Optional: specify goals
        #[arg(short = 'g', long)]
        goals: Option<String>,
        /// Optional: specify format (defaults to "text")
        #[arg(short = 'f', long, default_value = "text")]
        format: String,
        /// Optional: specify warnings (defaults to empty string)
        #[arg(short, long, default_value = "")]
        warnings: String,
        /// Optional: copy output to clipboard (defaults to false)
        #[arg(short = 'c', long)]
        clipboard: bool,
        /// Optional: suppress informational output for cleaner piping
        #[arg(short = 'q', long)]
        quiet: bool,
        /// Optional: read input from stdin (pipe) instead of interactive prompt
        #[arg(short = 'p', long)]
        pipe: bool,
        /// Hide thinking blocks (<think> </think>) and show an animation instead
        #[arg(short = 't', long)]
        no_thinking: bool,
        /// Enable recursion with specified number of waves (1-10)
        #[arg(short = 'r', long, value_parser = clap::value_parser!(u8).range(1..=10))]
        recursion: Option<u8>,
    },
    /// Demonstrates a friendly user prompt via dialoguer
    /// Configure LLM provider settings
    Configure {
        /// Optional: directly specify provider (skips interactive mode)
        #[arg(short, long)]
        provider: Option<String>,
        /// Optional: set API key (skips interactive prompt)
        #[arg(short, long)]
        api_key: Option<String>,
        /// Optional: specify model name
        #[arg(short, long)]
        model: Option<String>,
    },
    /// List available models for the configured provider
    Models {
        /// Optional: specify provider (defaults to configured provider)
        #[arg(short, long)]
        provider: Option<String>,
        /// Optional: suppress informational output, only show model names
        #[arg(short = 'q', long)]
        quiet: bool,
    },
    /// Run a session with specified goals, return format, and warnings.
    Session {
        /// Goals for the reasoning call
        #[arg(short, long, default_value = "")]
        goals: String,
        /// Expected return format
        #[arg(short = 'f', long)]
        return_format: String,
        /// Any warnings to consider
        #[arg(short, long, default_value = "")]
        warnings: String,
        /// Optional: suppress informational output for cleaner piping
        #[arg(short = 'q', long)]
        quiet: bool,
        /// Optional: read input from stdin (pipe) instead of interactive prompt
        #[arg(short = 'p', long)]
        pipe: bool,
    },
    /// Direct prompt without thinking steps structure
    NonThink {
        /// The raw prompt to send
        #[arg(short = 'p', long)]
        prompt: Option<String>,
        /// Optional: copy output to clipboard (defaults to false)
        #[arg(short = 'c', long)]
        clipboard: bool,
        /// Optional: suppress informational output for cleaner piping
        #[arg(short = 'q', long)]
        quiet: bool,
        /// Optional: read input from stdin (pipe) instead of interactive prompt
        #[arg(short = 'i', long)]
        pipe: bool,
        /// Filter out thinking blocks and show an animation instead
        #[arg(short = 'f', long)]
        filter_thinking: bool,
    },
    /// View or modify application settings
    Settings {
        /// Optional: View current settings
        #[arg(short, long)]
        view: bool,
        /// Optional: Set default model
        #[arg(long)]
        default_model: Option<String>,
        /// Optional: Set default return format
        #[arg(long)]
        default_format: Option<String>,
        /// Optional: Enable or disable logging
        #[arg(long)]
        logging: Option<bool>,
        /// Optional: Set log file location
        #[arg(long)]
        log_file: Option<String>,
        /// Optional: Reset settings to default values
        #[arg(short, long)]
        reset: bool,
    },
}

fn main() {
    let cli = OlaCli::parse();

    match &cli.command {
        Some(Commands::Start { verbose }) => {
            println!("Starting the application...");
            if *verbose {
                println!("Running in verbose mode!");
            }
            // Add custom logic here
        }
        Some(Commands::Prompt { goals, format, warnings, clipboard, quiet, pipe, no_thinking, recursion }) => {
            run_prompt(goals.clone(), &format, &warnings, *clipboard, *quiet, *pipe, *no_thinking, *recursion);
        }
        Some(Commands::NonThink { prompt, clipboard, quiet, pipe, filter_thinking }) => {
            run_non_think(prompt.clone(), *clipboard, *quiet, *pipe, *filter_thinking);
        }
        Some(Commands::Models { provider, quiet }) => {
            // Handle the Models subcommand
            list_models(provider.clone(), *quiet);
        }
        Some(Commands::Settings { view, default_model, default_format, logging, log_file, reset }) => {
            manage_settings(*view, default_model.clone(), default_format.clone(), *logging, log_file.clone(), *reset);
        }
        Some(Commands::Configure {
            provider: cli_provider,
            api_key: cli_api_key,
            model: cli_model,
        }) => {
            // Interactive configuration mode
            println!("🤖 Welcome to Ola Interactive Configuration!");

            // Provider selection - use command line arg if provided, otherwise ask
            let provider_name = if let Some(p) = cli_provider.clone() {
                p
            } else {
                let providers = vec!["OpenAI", "Anthropic", "Ollama"];
                let selected_idx = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Provider")
                    .items(&providers)
                    .default(0)
                    .interact()
                    .unwrap();
                providers[selected_idx].to_string()
            };

            // API Key handling based on provider and CLI args
            let api_key = if let Some(key) = cli_api_key.clone() {
                key
            } else {
                match provider_name.as_str() {
                    "Ollama" => {
                        println!("No API key needed for Ollama (using local instance)");
                        String::new()
                    }
                    _ => {
                        // Use Password input for secure API key entry
                        dialoguer::Password::with_theme(&ColorfulTheme::default())
                            .with_prompt(format!("{} API Key", provider_name))
                            .interact()
                            .unwrap()
                    }
                }
            };

            // Model selection - use CLI arg if provided
            let model = if let Some(m) = cli_model.clone() {
                Some(m)
            } else {
                match provider_name.as_str() {
                    "OpenAI" => {
                        let models = vec!["gpt-4o", "gpt-4-turbo", "gpt-4", "gpt-3.5-turbo"];
                        let idx = Select::with_theme(&ColorfulTheme::default())
                            .with_prompt("Model")
                            .items(&models)
                            .default(0)
                            .interact()
                            .unwrap();
                        Some(models[idx].to_string())
                    }
                    "Anthropic" => {
                        let models = vec![
                            "claude-3-opus-20240229",
                            "claude-3-sonnet-20240229",
                            "claude-3-haiku-20240307",
                            "claude-2.1",
                            "claude-2.0",
                        ];
                        let idx = Select::with_theme(&ColorfulTheme::default())
                            .with_prompt("Model")
                            .items(&models)
                            .default(0)
                            .interact()
                            .unwrap();
                        Some(models[idx].to_string())
                    }
                    "Ollama" => {
                        // Fetch available models from Ollama API
                        match config::fetch_ollama_models() {
                            Ok(models) => {
                                if models.is_empty() {
                                    eprintln!("No models found in Ollama. Using manual input...");
                                    let model: String = Input::with_theme(&ColorfulTheme::default())
                                        .with_prompt("Model name (e.g., llama2, mistral)")
                                        .default("llama2".into())
                                        .interact_text()
                                        .unwrap();
                                    Some(model)
                                } else {
                                    // Display available models in a select menu
                                    println!("Found {} models in Ollama", models.len());
                                    let selected_idx = Select::with_theme(&ColorfulTheme::default())
                                        .with_prompt("Select a model")
                                        .items(&models)
                                        .default(0)
                                        .interact()
                                        .unwrap();
                                    Some(models[selected_idx].clone())
                                }
                            },
                            Err(e) => {
                                eprintln!("Failed to fetch Ollama models: {}. Using manual input...", e);
                                let model: String = Input::with_theme(&ColorfulTheme::default())
                                    .with_prompt("Model name (e.g., llama2, mistral)")
                                    .default("llama2".into())
                                    .interact_text()
                                    .unwrap();
                                Some(model)
                            }
                        }
                    }
                    _ => None,
                }
            };

            // Create provider configuration
            let provider_config = config::ProviderConfig {
                provider: provider_name,
                api_key,
                model,
                additional_settings: None,
            };

            // Validate the configuration
            println!(
                "Validating configuration for provider: {}",
                provider_config.provider
            );
            if let Err(e) = config::validate_provider_config(&provider_config) {
                eprintln!("❌ Invalid configuration: {}", e);
                std::process::exit(1);
            }

            // Test connection if possible
            match provider_config.provider.as_str() {
                "Ollama" => {
                    println!("Testing connection to Ollama...");
                    // Simple test to check if Ollama is running
                    match std::process::Command::new("curl")
                        .arg("-s")
                        .arg("http://localhost:11434/api/version")
                        .output()
                    {
                        Ok(output) => {
                            if output.status.success() {
                                println!("✅ Successfully connected to Ollama");
                            } else {
                                eprintln!("❌ Failed to connect to Ollama. Is it running?");
                                std::process::exit(1);
                            }
                        }
                        Err(_) => {
                            eprintln!("❌ Failed to connect to Ollama. Is it running?");
                            std::process::exit(1);
                        }
                    }
                }
                "OpenAI" | "Anthropic" => {
                    println!(
                        "API key set for {}. Validation complete.",
                        provider_config.provider
                    );
                    // For Anthropic and OpenAI, we just check API key format in validate_provider_config
                    // A full API test would require making an actual API call
                }
                _ => {}
            };

            // Save configuration
            config::add_provider(provider_config.clone());
            if let Err(e) = config::save() {
                eprintln!("Failed to save configuration: {}", e);
                std::process::exit(1);
            }

            println!(
                "✅ Configuration saved for provider: {}",
                provider_config.provider
            );
            if let Some(model) = provider_config.model {
                println!("Using model: {}", model);
            }
        }
        Some(Commands::Session {
            goals,
            return_format,
            warnings,
            quiet,
            pipe,
        }) => {
            // If quiet mode is enabled, don't print informational messages
            if !quiet {
                eprintln!("Running session with the following parameters:");
                eprintln!("Goals: {}", goals);
                eprintln!("Return Format: {}", return_format);
                if !warnings.is_empty() {
                    eprintln!("Warnings: {}", warnings);
                }
            }
            
            // Check if we should use stdin input
            let input_content = if *pipe {
                read_from_stdin()
            } else {
                String::new()
            };
            
            // In a real app, you'd pass input_content to the reasoning model
            let output = if input_content.is_empty() {
                format!("Processed session for goals: {}", goals)
            } else {
                format!("Processed session for goals: {} with input: {}", goals, input_content)
            };
            
            // Send the main output to stdout for piping
            println!("{}", output);

            // Log session output to a jsonl file
            let log_entry = json!({
                "timestamp": Utc::now().to_rfc3339(),
                "goals": goals,
                "return_format": return_format,
                "warnings": warnings,
                "input": input_content,
                "output": output,
            });
            if let Err(e) = append_to_log("sessions.jsonl", &log_entry.to_string()) {
                eprintln!("Failed to log session: {}", e);
            } else if !quiet {
                eprintln!("Session output logged to sessions.jsonl");
            }
        }
        None => {
            eprintln!("No subcommand was used. Try `ola --help` for more info.");
        }
    }
}

fn read_from_stdin() -> String {
    utils::piping::read_from_stdin()
}

fn run_prompt(cli_goals: Option<String>, cli_format: &str, cli_warnings: &str, clipboard: bool, quiet: bool, pipe: bool, no_thinking: bool, recursion: Option<u8>) {
    // Track recursion wave number (defaults to 0 for non-recursive operations)
    let wave_number = std::env::var("OLA_RECURSION_WAVE").ok().and_then(|s| s.parse::<u8>().ok()).unwrap_or(0);
    
    // Log the current recursion wave if recursion is enabled
    if wave_number > 0 && !quiet {
        // Define ocean-themed colors for different waves
        let wave_colors = [
            "\x1b[34m",  // blue
            "\x1b[36m",  // cyan
            "\x1b[96m",  // bright cyan
            "\x1b[94m",  // bright blue
            "\x1b[38;5;39m",  // deep sky blue
            "\x1b[38;5;45m",  // turquoise
            "\x1b[38;5;23m",  // sea green
            "\x1b[38;5;24m",  // deep turquoise
            "\x1b[38;5;31m",  // medium blue
            "\x1b[38;5;37m",  // teal
        ];
        
        let color = wave_colors[(wave_number as usize - 1) % wave_colors.len()];
        let reset = "\x1b[0m";
        
        eprintln!("{}[RECURSION WAVE {}]{}  Processing...", color, wave_number, reset);
    } else if !quiet {
        eprintln!("Welcome to the Ola CLI Prompt!");
    }
    
    // Read from stdin if pipe mode is enabled
    let piped_content = if pipe {
        read_from_stdin()
    } else {
        String::new()
    };

    // Check if goals were provided via CLI to determine flow
    let cli_goals_provided = cli_goals.is_some();
    
    // Get goals from CLI args or prompt user
    let goals = if let Some(ref g) = cli_goals {
        g.clone()
    } else if !piped_content.is_empty() {
        // Use piped content as goals if no explicit goals were provided
        piped_content.clone()
    } else {
        Input::with_theme(&ColorfulTheme::default())
            .with_prompt("🏆 Goals: ")
            .default("Anonymous".into())
            .interact_text()
            .unwrap()
    };

    // If goals were provided via CLI, use the CLI args for format and warnings too
    // Otherwise, prompt for all three parts
    let (format, warnings) = if cli_goals_provided || !piped_content.is_empty() {
        (cli_format.to_string(), cli_warnings.to_string())
    } else {
        // Prompt for return format
        let format = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("📝 Return Format: ")
            .default("text".into())
            .interact_text()
            .unwrap();
        
        // Prompt for warnings
        let warnings = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("⚠️ Warnings: ")
            .default("".into())
            .interact_text()
            .unwrap();
        
        (format, warnings)
    };

    // If we have piped content but also explicit goals, use piped content as context
    let (final_goals, context) = if !piped_content.is_empty() && cli_goals_provided {
        (goals, Some(piped_content))
    } else {
        (goals, None)
    };

    // Call the prompt function from the ola crate with context
    let output = match &context {
        Some(ctx) => prompt::structure_reasoning(&final_goals, &format, &warnings, clipboard, Some(ctx), no_thinking),
        None => prompt::structure_reasoning(&final_goals, &format, &warnings, clipboard, None, no_thinking),
    };

    if !quiet {
        eprintln!(
            "Goals: {}\nReturn Format: {}\nWarnings: {}",
            final_goals, format, warnings
        );
        if let Some(ctx) = context {
            eprintln!("Context from stdin: {} characters", ctx.len());
        }
    }
    
    match output {
        Ok(()) => {
            if !quiet {
                eprintln!("Prompt executed successfully");
            }
            
            // Handle recursion if enabled and we haven't reached the limit
            if let Some(max_waves) = recursion {
                if wave_number < max_waves {
                    // Prepare to launch the next recursion wave
                    let next_wave = wave_number + 1;
                    
                    if !quiet {
                        eprintln!("Launching recursion wave {}...", next_wave);
                    }
                    
                    // Build the command to execute the next wave
                    let current_exe = std::env::current_exe().expect("Failed to get current executable path");
                    
                    // Create a new Command instance using the current executable
                    let mut cmd = std::process::Command::new(current_exe);
                    
                    // Set the OLA_RECURSION_WAVE environment variable for the child process
                    cmd.env("OLA_RECURSION_WAVE", next_wave.to_string());
                    
                    // Add the "prompt" subcommand
                    cmd.arg("prompt");
                    
                    // Add all the original arguments
                    if let Some(g) = &cli_goals {
                        cmd.args(["--goals", g]);
                    }
                    cmd.args(["--format", cli_format]);
                    if !cli_warnings.is_empty() {
                        cmd.args(["--warnings", cli_warnings]);
                    }
                    if clipboard {
                        cmd.arg("--clipboard");
                    }
                    if quiet {
                        cmd.arg("--quiet");
                    }
                    if pipe {
                        cmd.arg("--pipe");
                    }
                    if no_thinking {
                        cmd.arg("--no-thinking");
                    }
                    cmd.args(["--recursion", &max_waves.to_string()]);
                    
                    // Execute the command
                    match cmd.status() {
                        Ok(status) => {
                            if !status.success() {
                                eprintln!("Recursion wave {} failed with status: {}", next_wave, status);
                            }
                        },
                        Err(e) => {
                            eprintln!("Failed to launch recursion wave {}: {}", next_wave, e);
                        }
                    }
                } else if !quiet {
                    eprintln!("Reached maximum recursion depth ({} waves)", max_waves);
                }
            }
        },
        Err(e) => eprintln!("Prompt returned error: {:?}", e),
    }
}

fn run_non_think(cli_prompt: Option<String>, clipboard: bool, quiet: bool, pipe: bool, filter_thinking: bool) {
    if !quiet {
        eprintln!("Running direct prompt without thinking steps...");
    }

    // Read from stdin if pipe mode is enabled
    let piped_content = if pipe {
        read_from_stdin()
    } else {
        String::new()
    };

    // Check if prompt was provided via CLI to determine flow
    let cli_prompt_provided = cli_prompt.is_some();
    
    // Get prompt from CLI args or prompt user
    let prompt = if let Some(p) = cli_prompt {
        p
    } else if !piped_content.is_empty() {
        // Use piped content as prompt if no explicit prompt was provided
        piped_content.clone()
    } else {
        Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter your prompt: ")
            .default("".into())
            .interact_text()
            .unwrap()
    };

    // If we have piped content but also explicit prompt, use piped content as context
    let (final_prompt, context) = if !piped_content.is_empty() && cli_prompt_provided {
        (prompt, Some(piped_content))
    } else {
        (prompt, None)
    };

    // Call the new function from the prompt module
    let output = match &context {
        Some(ctx) => prompt::stream_non_think(&final_prompt, clipboard, Some(ctx), filter_thinking),
        None => prompt::stream_non_think(&final_prompt, clipboard, None, filter_thinking),
    };

    if !quiet {
        if let Some(ctx) = context {
            eprintln!("Context from stdin: {} characters", ctx.len());
        }
    }
    
    match output {
        Ok(()) => {
            if !quiet {
                eprintln!("Non-think prompt executed successfully");
            }
        },
        Err(e) => eprintln!("Prompt returned error: {:?}", e),
    }
}

fn append_to_log(filename: &str, entry: &str) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(filename)?;
    writeln!(file, "{}", entry)?;
    Ok(())
}

/// Manage application settings
fn manage_settings(
    view: bool, 
    default_model: Option<String>, 
    default_format: Option<String>,
    logging: Option<bool>,
    log_file: Option<String>,
    reset: bool
) {
    // Try to load existing settings
    let mut settings = match settings::Settings::load() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to load settings: {}", e);
            if reset || default_model.is_some() || default_format.is_some() || 
               logging.is_some() || log_file.is_some() {
                // Create default settings if we need to modify them
                settings::Settings::default()
            } else {
                // Just exit if we're only trying to view
                std::process::exit(1);
            }
        }
    };
    
    // Reset settings if requested
    if reset {
        settings = settings::Settings::default();
        println!("Settings reset to default values");
    }
    
    // Update settings with provided values
    if let Some(ref model) = default_model {
        settings.default_model = model.clone();
        println!("Default model set to: {}", settings.default_model);
    }
    
    if let Some(ref format) = default_format {
        settings.defaults.return_format = format.clone();
        println!("Default return format set to: {}", settings.defaults.return_format);
    }
    
    if let Some(enable_logging) = logging {
        settings.behavior.enable_logging = enable_logging;
        println!("Logging is now {}", if enable_logging { "enabled" } else { "disabled" });
    }
    
    if let Some(ref file) = log_file {
        settings.behavior.log_file = file.clone();
        println!("Log file set to: {}", settings.behavior.log_file);
    }
    
    // Save settings if any changes were made
    if reset || default_model.is_some() || default_format.is_some() || 
       logging.is_some() || log_file.is_some() {
        if let Err(e) = settings.save() {
            eprintln!("Failed to save settings: {}", e);
            std::process::exit(1);
        } else {
            println!("Settings saved successfully to: ~/.ola/settings.yaml");
        }
    }
    
    // View settings if requested or if no other options were provided
    if view || (!reset && default_model.is_none() && default_format.is_none() && 
        logging.is_none() && log_file.is_none()) {
        // Convert settings to YAML for display
        match serde_yaml::to_string(&settings) {
            Ok(yaml) => {
                println!("Current settings:\n{}", yaml);
            },
            Err(e) => {
                eprintln!("Failed to serialize settings: {}", e);
                std::process::exit(1);
            }
        }
    }
}

/// List available models for the specified provider
fn list_models(provider: Option<String>, quiet: bool) {
    // Load current configuration
    let config = match config::Config::load() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };

    // Determine the provider to use
    let provider_name = if let Some(p) = provider {
        p
    } else if !config.active_provider.is_empty() {
        config.active_provider.clone()
    } else {
        eprintln!("No provider specified and no active provider configured.");
        eprintln!("Please run 'ola configure' first or specify a provider with --provider.");
        std::process::exit(1);
    };

    if !quiet {
        println!("Fetching available models for provider: {}", provider_name);
    }

    match provider_name.as_str() {
        "Ollama" => {
            // Fetch models from Ollama API
            match config::fetch_ollama_models() {
                Ok(models) => {
                    if models.is_empty() {
                        if !quiet {
                            println!("No models found in Ollama.");
                        }
                    } else {
                        if !quiet {
                            println!("Available Ollama models:");
                            for (i, model) in models.iter().enumerate() {
                                println!("  {}. {}", i + 1, model);
                            }
                        } else {
                            // In quiet mode, just print model names (one per line)
                            for model in models {
                                println!("{}", model);
                            }
                        }
                    }
                },
                Err(e) => {
                    eprintln!("Failed to fetch Ollama models: {}", e);
                    eprintln!("Is Ollama running on http://localhost:11434?");
                    std::process::exit(1);
                }
            }
        },
        "OpenAI" => {
            if !quiet {
                println!("OpenAI models:");
                println!("  1. gpt-4o");
                println!("  2. gpt-4-turbo");
                println!("  3. gpt-4");
                println!("  4. gpt-3.5-turbo");
            } else {
                println!("gpt-4o");
                println!("gpt-4-turbo");
                println!("gpt-4");
                println!("gpt-3.5-turbo");
            }
        },
        "Anthropic" => {
            if !quiet {
                println!("Anthropic models:");
                println!("  1. claude-3-opus-20240229");
                println!("  2. claude-3-sonnet-20240229");
                println!("  3. claude-3-haiku-20240307");
                println!("  4. claude-2.1");
                println!("  5. claude-2.0");
            } else {
                println!("claude-3-opus-20240229");
                println!("claude-3-sonnet-20240229");
                println!("claude-3-haiku-20240307");
                println!("claude-2.1");
                println!("claude-2.0");
            }
        },
        _ => {
            eprintln!("Unsupported provider: {}", provider_name);
            std::process::exit(1);
        }
    }
}
