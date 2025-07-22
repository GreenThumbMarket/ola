/*
.olaHints content:
- This file contains hints for optimizing model calls.
- Use the format [Goals, Return Format, Warnings] when configuring sessions.
- Providers and session storage are easily configurable.
*/

use chrono::Utc;
use clap::Parser;
use dialoguer::{theme::ColorfulTheme, Input, Select, Confirm};
use serde_json::json;
use std::fs::OpenOptions;
use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

// Core modules
mod config;
mod prompt;
mod settings;
mod models;
mod project;

// API communication layer
mod api;

// Utility modules
mod utils;

#[derive(Parser)]
#[command(name = "ola")]
#[command(version = "0.2.0")]
#[command(about = "A friendly CLI for prompting and optimizing reasoning model calls. Use without subcommand for default prompt behavior.", long_about = None)]
struct OlaCli {
    /// Optional: specify goals (when no subcommand provided)
    #[arg(short = 'g', long)]
    goals: Option<String>,
    /// Optional: specify format (defaults to "text")
    #[arg(short = 'f', long, default_value = "text")]
    format: Option<String>,
    /// Optional: specify warnings (defaults to empty string)
    #[arg(short, long, default_value = "")]
    warnings: Option<String>,
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
    /// Enable interactive iteration mode with user feedback between iterations (1-10)
    #[arg(short = 'i', long, value_parser = clap::value_parser!(u8).range(1..=10))]
    iterations: Option<u8>,
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
        /// Enable interactive iteration mode with user feedback between iterations (1-10)
        #[arg(short = 'i', long, value_parser = clap::value_parser!(u8).range(1..=10))]
        iterations: Option<u8>,
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
    /// Project management commands  
    Project {
        #[command(subcommand)]
        command: Option<ProjectCommands>,
    },
}

#[derive(clap::Subcommand)]
enum ProjectCommands {
    /// List all projects (default action)
    #[command(alias = "ls")]
    List,
    /// Create a new project
    Create {
        /// Project name (optional, will prompt if not provided)
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Delete a project
    #[command(alias = "rm")]
    Delete {
        /// Project name to delete (optional, will prompt if not provided)
        #[arg(short, long)]
        project: Option<String>,
        /// Force deletion without confirmation
        #[arg(short, long)]
        force: bool,
    },
    /// Edit project details
    Edit {
        /// Project name to edit (optional, will prompt if not provided)
        #[arg(short, long)]
        project: Option<String>,
        /// New project name
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Set active project
    Set {
        /// Project name to set as active (optional, will prompt if not provided)
        #[arg(short, long)]
        project: Option<String>,
    },
    /// Show project details
    Show {
        /// Project name (optional, uses active if not specified)
        #[arg(short, long)]
        project: Option<String>,
    },
    /// Upload a file to a project
    Upload {
        /// Project ID (optional, uses active if not specified)
        #[arg(short, long)]
        project: Option<String>,
        /// File path to upload
        #[arg(short, long)]
        file: String,
    },
    /// List files in a project
    Files {
        /// Project ID (optional, uses active if not specified)
        #[arg(short, long)]
        project: Option<String>,
    },
    /// Add a goal to a project
    AddGoal {
        /// Project ID (optional, uses active if not specified)
        #[arg(short, long)]
        project: Option<String>,
        /// Goal text
        #[arg(short, long)]
        goal: String,
    },
    /// Remove a goal from a project
    RemoveGoal {
        /// Project ID (optional, uses active if not specified)
        #[arg(short, long)]
        project: Option<String>,
        /// Goal ID to remove
        #[arg(short, long)]
        goal_id: String,
    },
    /// Add context to a project
    AddContext {
        /// Project ID (optional, uses active if not specified)
        #[arg(short, long)]
        project: Option<String>,
        /// Context text
        #[arg(short, long)]
        context: String,
    },
    /// Remove context from a project
    RemoveContext {
        /// Project ID (optional, uses active if not specified)
        #[arg(short, long)]
        project: Option<String>,
        /// Context ID to remove
        #[arg(short, long)]
        context_id: String,
    },
    /// Remove a file from a project
    RemoveFile {
        /// Project ID (optional, uses active if not specified)
        #[arg(short, long)]
        project: Option<String>,
        /// File ID to remove
        #[arg(short, long)]
        file_id: String,
    },
    /// Run a prompt with project context
    Run {
        /// Project name (optional, uses active if not specified)
        #[arg(short, long)]
        project: Option<String>,
        /// Prompt text
        #[arg(short = 'g', long)]
        goals: String,
        /// Return format
        #[arg(short = 'f', long, default_value = "text")]
        format: String,
        /// Warnings
        #[arg(short, long, default_value = "")]
        warnings: String,
        /// Copy to clipboard
        #[arg(short = 'c', long)]
        clipboard: bool,
        /// Hide thinking blocks
        #[arg(short = 't', long)]
        no_thinking: bool,
    },
}

fn main() {
    let cli = OlaCli::parse();

    // If no subcommand is provided, use the default prompt behavior
    match &cli.command {
        None => {
            // Default to prompt command with CLI args
            run_prompt(
                cli.goals.clone(),
                &cli.format.unwrap_or_else(|| "text".to_string()),
                &cli.warnings.unwrap_or_else(|| "".to_string()),
                cli.clipboard,
                cli.quiet,
                cli.pipe,
                cli.no_thinking,
                cli.recursion,
                cli.iterations,
            );
        }
        Some(Commands::Start { verbose }) => {
            utils::output::startup_animation();
            utils::output::print_success("Application started successfully!");
            if *verbose {
                utils::output::println_colored("Running in verbose mode!", utils::output::Color::BrightYellow);
            }
            // Add custom logic here
        }
        Some(Commands::Prompt { goals, format, warnings, clipboard, quiet, pipe, no_thinking, recursion, iterations }) => {
            run_prompt(goals.clone(), format, warnings, *clipboard, *quiet, *pipe, *no_thinking, *recursion, *iterations);
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
        Some(Commands::Project { command }) => {
            handle_project_command(command.as_ref().unwrap_or(&ProjectCommands::List));
        }
        Some(Commands::Configure {
            provider: cli_provider,
            api_key: cli_api_key,
            model: cli_model,
        }) => {
            // Interactive configuration mode with colorful banner
            utils::output::print_banner("🤖 Welcome to Ola Interactive Configuration! 🤖", utils::output::Color::DeepSkyBlue);

            // Check for auto-detection from environment variables first
            if let Some(detected_config) = config::detect_provider_from_env() {
                println!("🔍 Auto-detected configuration from environment variables:");
                println!("   Provider: {}", detected_config.provider);
                println!("   Model: {}", detected_config.model.as_ref().unwrap_or(&"default".to_string()));
                
                let confirm = Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt("Use this configuration?")
                    .default(true)
                    .interact()
                    .unwrap();
                
                if confirm {
                    // Validate the auto-detected configuration
                    println!("Validating auto-detected configuration...");
                    if let Err(e) = config::validate_provider_config(&detected_config) {
                        eprintln!("❌ Invalid auto-detected configuration: {}", e);
                        std::process::exit(1);
                    }
                    
                    // Save auto-detected configuration
                    config::add_provider(detected_config.clone());
                    if let Err(e) = config::save() {
                        eprintln!("Failed to save configuration: {}", e);
                        std::process::exit(1);
                    }
                    
                    println!("✅ Auto-detected configuration saved for provider: {}", detected_config.provider);
                    if let Some(model) = detected_config.model {
                        println!("Using model: {}", model);
                    }
                    return;
                }
            }

            // Provider selection - use command line arg if provided, otherwise ask
            let provider_name = if let Some(p) = cli_provider.clone() {
                p
            } else {
                let providers = vec!["OpenAI", "Anthropic", "Ollama", "Gemini"];
                let selected_idx = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Provider")
                    .items(&providers)
                    .default(0)
                    .interact()
                    .unwrap();
                providers[selected_idx].to_string()
            };

            // API Key handling - check environment first, then CLI args, then prompt
            let api_key = if let Some(key) = cli_api_key.clone() {
                key
            } else {
                // Check environment variables first
                let env_key = match provider_name.as_str() {
                    "OpenAI" => std::env::var("OPENAI_API_KEY").ok(),
                    "Anthropic" => std::env::var("ANTHROPIC_API_KEY").ok(),
                    "Gemini" => std::env::var("GEMINI_API_KEY").ok(),
                    _ => None,
                };
                
                if let Some(key) = env_key {
                    if !key.trim().is_empty() {
                        println!("🔍 Using API key from environment variable");
                        key
                    } else {
                        // Prompt for API key if env var is empty
                        match provider_name.as_str() {
                            "Ollama" => {
                                println!("No API key needed for Ollama (using local instance)");
                                String::new()
                            }
                            "Gemini" => {
                                println!("For Gemini, you need an API key from Google AI Studio (https://aistudio.google.com/)");
                                dialoguer::Password::with_theme(&ColorfulTheme::default())
                                    .with_prompt("Google API Key")
                                    .interact()
                                    .unwrap()
                            }
                            _ => {
                                dialoguer::Password::with_theme(&ColorfulTheme::default())
                                    .with_prompt(format!("{} API Key", provider_name))
                                    .interact()
                                    .unwrap()
                            }
                        }
                    }
                } else {
                    // No env var found, prompt for API key
                    match provider_name.as_str() {
                        "Ollama" => {
                            println!("No API key needed for Ollama (using local instance)");
                            String::new()
                        }
                        "Gemini" => {
                            println!("For Gemini, you need an API key from Google AI Studio (https://aistudio.google.com/)");
                            dialoguer::Password::with_theme(&ColorfulTheme::default())
                                .with_prompt("Google API Key")
                                .interact()
                                .unwrap()
                        }
                        _ => {
                            dialoguer::Password::with_theme(&ColorfulTheme::default())
                                .with_prompt(format!("{} API Key", provider_name))
                                .interact()
                                .unwrap()
                        }
                    }
                }
            };

            // Model selection - use CLI arg if provided
            let model = if let Some(m) = cli_model.clone() {
                Some(m)
            } else {
                match provider_name.as_str() {
                    "OpenAI" => {
                        let models = vec!["gpt-4o", "gpt-4", "o3", "o3-pro", "o4", "o4-mini", "o4-mini-high"];
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
                    "Gemini" => {
                        let models = vec![
                            "gemini-1.5-pro",
                            "gemini-1.5-flash",
                            "gemini-1.0-pro",
                            "gemini-1.0-pro-vision",
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
                                    utils::output::println_colored("🔍 No models found in Ollama. Using manual input...", utils::output::Color::Orange);
                                    let model: String = Input::with_theme(&ColorfulTheme::default())
                                        .with_prompt("Model name (e.g., llama2, mistral)")
                                        .default("llama2".into())
                                        .interact_text()
                                        .unwrap();
                                    Some(model)
                                } else {
                                    // Display available models in a select menu
                                    utils::output::println_colored(&format!("✨ Found {} models in Ollama", models.len()), utils::output::Color::BrightGreen);
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
            utils::output::print_spinner_frame(0, &format!("Validating configuration for provider: {}", provider_config.provider));
            if let Err(e) = config::validate_provider_config(&provider_config) {
                eprintln!("❌ Invalid configuration: {}", e);
                std::process::exit(1);
            }

            // Test connection if possible
            match provider_config.provider.as_str() {
                "Ollama" => {
                    utils::output::println_colored("🔌 Testing connection to Ollama...", utils::output::Color::BrightCyan);
                    // Simple test to check if Ollama is running
                    match std::process::Command::new("curl")
                        .arg("-s")
                        .arg("http://localhost:11434/api/version")
                        .output()
                    {
                        Ok(output) => {
                            if output.status.success() {
                                utils::output::clear_line();
                                utils::output::print_success("Successfully connected to Ollama");
                            } else {
                                utils::output::clear_line();
                                utils::output::print_error("Failed to connect to Ollama. Is it running?");
                                std::process::exit(1);
                            }
                        }
                        Err(_) => {
                            utils::output::clear_line();
                            utils::output::print_error("Failed to connect to Ollama. Is it running?");
                            std::process::exit(1);
                        }
                    }
                }
                "OpenAI" | "Anthropic" => {
                    utils::output::print_success(&format!(
                        "API key set for {}. Validation complete.",
                        provider_config.provider
                    ));
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

            utils::output::print_success(&format!(
                "Configuration saved for provider: {}",
                provider_config.provider
            ));
            if let Some(model) = provider_config.model {
                utils::output::println_colored(&format!("🧠 Using model: {}", model), utils::output::Color::BrightBlue);
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
    }
}

fn read_from_stdin() -> String {
    utils::piping::read_from_stdin()
}

fn run_prompt(cli_goals: Option<String>, cli_format: &str, cli_warnings: &str, clipboard: bool, quiet: bool, pipe: bool, no_thinking: bool, recursion: Option<u8>, iterations: Option<u8>) {
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
        utils::output::print_rainbow("🌊 Welcome to the Ola CLI Prompt! 🌊");
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

    // Call the appropriate function based on whether iterations are enabled
    let output = if let Some(max_iterations) = iterations {
        // Use iteration mode
        prompt::interactive_iterations(&final_goals, &format, &warnings, clipboard, context.as_deref(), no_thinking, max_iterations)
    } else {
        // Use standard reasoning
        match &context {
            Some(ctx) => prompt::structure_reasoning(&final_goals, &format, &warnings, clipboard, Some(ctx), no_thinking),
            None => prompt::structure_reasoning(&final_goals, &format, &warnings, clipboard, None, no_thinking),
        }
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
                    if let Some(iter) = iterations {
                        cmd.args(["--iterations", &iter.to_string()]);
                    }
                    
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
        utils::output::print_spinner_frame(0, &format!("Fetching available models for provider: {}", provider_name));
        std::thread::sleep(std::time::Duration::from_millis(500));
        utils::output::clear_line();
    }

    match provider_name.as_str() {
        "Ollama" => {
            // Fetch models from Ollama API
            match config::fetch_ollama_models() {
                Ok(models) => {
                    if models.is_empty() {
                        if !quiet {
                            utils::output::println_colored("🔍 No models found in Ollama.", utils::output::Color::Orange);
                        }
                    } else {
                        if !quiet {
                            utils::output::print_banner("🤖 Available Ollama Models 🤖", utils::output::Color::BrightGreen);
                            for (i, model) in models.iter().enumerate() {
                                utils::output::println_colored(&format!("  {}. {}", i + 1, model), utils::output::Color::BrightCyan);
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
                utils::output::print_banner("🧠 OpenAI Models 🧠", utils::output::Color::BrightGreen);
                utils::output::println_colored("  1. gpt-4o", utils::output::Color::BrightCyan);
                utils::output::println_colored("  2. gpt-4", utils::output::Color::BrightCyan);
                utils::output::println_colored("  3. o3", utils::output::Color::BrightCyan);
                utils::output::println_colored("  4. o3-pro", utils::output::Color::BrightCyan);
                utils::output::println_colored("  5. o4", utils::output::Color::BrightCyan);
                utils::output::println_colored("  6. o4-mini", utils::output::Color::BrightCyan);
                utils::output::println_colored("  7. o4-mini-high", utils::output::Color::BrightCyan);
            } else {
                println!("gpt-4o");
                println!("gpt-4");
                println!("o3");
                println!("o3-pro");
                println!("o4");
                println!("o4-mini");
                println!("o4-mini-high");
            }
        },
        "Gemini" => {
            if !quiet {
                utils::output::print_banner("💎 Google Gemini Models 💎", utils::output::Color::Purple);
                utils::output::println_colored("  1. gemini-1.5-pro", utils::output::Color::BrightCyan);
                utils::output::println_colored("  2. gemini-1.5-flash", utils::output::Color::BrightCyan);
                utils::output::println_colored("  3. gemini-1.0-pro", utils::output::Color::BrightCyan);
                utils::output::println_colored("  4. gemini-1.0-pro-vision", utils::output::Color::BrightCyan);
            } else {
                println!("gemini-1.5-pro");
                println!("gemini-1.5-flash");
                println!("gemini-1.0-pro");
                println!("gemini-1.0-pro-vision");
            }
        },
        "Anthropic" => {
            if !quiet {
                utils::output::print_banner("🎭 Anthropic Claude Models 🎭", utils::output::Color::Orange);
                utils::output::println_colored("  1. claude-3-opus-20240229", utils::output::Color::BrightCyan);
                utils::output::println_colored("  2. claude-3-sonnet-20240229", utils::output::Color::BrightCyan);
                utils::output::println_colored("  3. claude-3-haiku-20240307", utils::output::Color::BrightCyan);
                utils::output::println_colored("  4. claude-2.1", utils::output::Color::BrightCyan);
                utils::output::println_colored("  5. claude-2.0", utils::output::Color::BrightCyan);
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

/// Handle project management commands
fn handle_project_command(command: &ProjectCommands) {
    use project::ProjectManager;
    use models::{Goal, Context};
    
    let project_manager = match ProjectManager::new() {
        Ok(pm) => pm,
        Err(e) => {
            eprintln!("Failed to initialize project manager: {}", e);
            std::process::exit(1);
        }
    };

    // Helper function to resolve project name/ID with guided selection
    let resolve_project_with_guidance = |project_name: Option<&String>, action_description: &str| -> anyhow::Result<String> {
        match project_name {
            Some(name) => {
                // Find project by name
                let projects = project_manager.list_projects()?;
                if let Some(proj) = projects.iter().find(|p| p.name.eq_ignore_ascii_case(name)) {
                    Ok(proj.id.clone())
                } else {
                    Err(anyhow::anyhow!("Project '{}' not found", name))
                }
            }
            None => {
                // Interactive project selection
                let projects = project_manager.list_projects()?;
                if projects.is_empty() {
                    return Err(anyhow::anyhow!("No projects available. Create one first with 'ola project create --name <name>'"));
                }
                
                let project_names: Vec<String> = projects.iter().map(|p| {
                    let active_marker = if let Ok(Some(active_id)) = project_manager.get_active_project() {
                        if active_id == p.id { " (active)" } else { "" }
                    } else { "" };
                    format!("{}{}", p.name, active_marker)
                }).collect();
                
                let selected_idx = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt(&format!("Select project to {}", action_description))
                    .items(&project_names)
                    .default(0)
                    .interact()
                    .map_err(|e| anyhow::anyhow!("Selection failed: {}", e))?;
                
                Ok(projects[selected_idx].id.clone())
            }
        }
    };

    match command {
        ProjectCommands::List => {
            let active_project_id = project_manager.get_active_project().unwrap_or(None);
            
            match project_manager.list_projects() {
                Ok(projects) => {
                    if projects.is_empty() {
                        println!("No projects found. Create one with 'ola project create --name <name>'");
                    } else {
                        println!("Projects:");
                        
                        let mut stdout = StandardStream::stdout(ColorChoice::Auto);
                        
                        for project in projects {
                            let is_active = active_project_id.as_ref() == Some(&project.id);
                            
                            if is_active {
                                let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)).set_bold(true));
                                print!("* ");
                            } else {
                                let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)));
                                print!("  ");
                            }
                            
                            println!("{} - {} ({} files, {} goals, {} contexts)", 
                                   project.id, 
                                   project.name, 
                                   project.files.len(),
                                   project.goals.len(),
                                   project.contexts.len());
                            
                            let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_dimmed(true));
                            println!("    Updated: {}", project.updated_at.format("%Y-%m-%d %H:%M:%S"));
                            let _ = stdout.reset();
                        }
                        
                        if let Some(active_id) = active_project_id {
                            let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)).set_dimmed(true));
                            println!("\nActive project: {}", active_id);
                            let _ = stdout.reset();
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to list projects: {}", e);
                    std::process::exit(1);
                }
            }
        }
        
        ProjectCommands::Create { name } => {
            println!("🚀 Welcome to Ola Project Creation!");
            
            // Get project name - from CLI arg or prompt
            let project_name = match name {
                Some(n) => n.clone(),
                None => {
                    Input::with_theme(&ColorfulTheme::default())
                        .with_prompt("Project name")
                        .interact_text()
                        .map_err(|e| {
                            eprintln!("Input failed: {}", e);
                            std::process::exit(1);
                        })
                        .unwrap()
                }
            };
            
            // Check if project with this name already exists
            let existing_projects = match project_manager.list_projects() {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Failed to check existing projects: {}", e);
                    std::process::exit(1);
                }
            };
            
            if existing_projects.iter().any(|p| p.name.eq_ignore_ascii_case(&project_name)) {
                eprintln!("A project named '{}' already exists. Please choose a different name.", project_name);
                std::process::exit(1);
            }
            
            match project_manager.create_project(project_name.clone()) {
                Ok(project) => {
                    println!("✅ Created project '{}' with ID: {}", project.name, project.id);
                    println!("   Project directory: ~/.ola/data/projects/{}", project.id);
                    
                    // Set as active project if no active project is set or if user confirms
                    let should_set_active = if project_manager.get_active_project().unwrap_or(None).is_none() {
                        true
                    } else {
                        Confirm::with_theme(&ColorfulTheme::default())
                            .with_prompt(&format!("Set '{}' as active project?", project.name))
                            .default(true)
                            .interact()
                            .unwrap_or(false)
                    };
                    
                    if should_set_active {
                        if let Err(e) = project_manager.set_active_project(&project.id) {
                            eprintln!("Warning: Failed to set as active project: {}", e);
                        } else {
                            println!("   Set as active project");
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to create project: {}", e);
                    std::process::exit(1);
                }
            }
        }

        ProjectCommands::Delete { project, force } => {
            let project_id = match resolve_project_with_guidance(project.as_ref(), "delete") {
                Ok(id) => id,
                Err(e) => {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
            };
            
            // Get project details for confirmation
            let project_name = match project_manager.load_project(&project_id) {
                Ok(Some(proj)) => proj.name,
                Ok(None) => {
                    eprintln!("Project not found");
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("Failed to load project: {}", e);
                    std::process::exit(1);
                }
            };
            
            if !force {
                let confirmation = Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt(&format!("Are you sure you want to delete project '{}'? This cannot be undone.", project_name))
                    .default(false)
                    .interact()
                    .unwrap();
                
                if !confirmation {
                    println!("Deletion cancelled");
                    return;
                }
            }
            
            match project_manager.delete_project(&project_id) {
                Ok(_) => {
                    println!("✅ Deleted project '{}'", project_name);
                    
                    // Clear active project if it was the deleted one
                    if let Ok(Some(active)) = project_manager.get_active_project() {
                        if active == project_id {
                            let active_file = std::env::var("HOME")
                                .map(|h| std::path::PathBuf::from(h).join(".ola").join("active_project"))
                                .unwrap_or_default();
                            let _ = std::fs::remove_file(&active_file);
                            println!("   Cleared as active project");
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to delete project: {}", e);
                    std::process::exit(1);
                }
            }
        }
        
        ProjectCommands::Edit { project, name } => {
            let project_id = match resolve_project_with_guidance(project.as_ref(), "edit") {
                Ok(id) => id,
                Err(e) => {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
            };
            
            // Get current project details
            let current_project = match project_manager.load_project(&project_id) {
                Ok(Some(proj)) => proj,
                Ok(None) => {
                    eprintln!("Project not found");
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("Failed to load project: {}", e);
                    std::process::exit(1);
                }
            };
            
            let old_name = current_project.name.clone();
            
            // Get new name - from CLI arg or prompt
            let new_name = match name {
                Some(n) => n.clone(),
                None => {
                    Input::with_theme(&ColorfulTheme::default())
                        .with_prompt("New project name")
                        .default(old_name.clone())
                        .interact_text()
                        .map_err(|e| {
                            eprintln!("Input failed: {}", e);
                            std::process::exit(1);
                        })
                        .unwrap()
                }
            };
            
            if new_name != old_name {
                match project_manager.edit_project(&project_id, Some(new_name.clone())) {
                    Ok(_) => {
                        println!("✅ Updated project name from '{}' to '{}'", old_name, new_name);
                    }
                    Err(e) => {
                        eprintln!("Failed to edit project: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                println!("No changes made to project '{}'", old_name);
            }
        }
        
        ProjectCommands::Set { project } => {
            let project_id = match resolve_project_with_guidance(project.as_ref(), "set as active") {
                Ok(id) => id,
                Err(e) => {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
            };
            
            let project_name = match project_manager.load_project(&project_id) {
                Ok(Some(proj)) => proj.name,
                Ok(None) => {
                    eprintln!("Project not found");
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("Failed to load project: {}", e);
                    std::process::exit(1);
                }
            };
            
            match project_manager.set_active_project(&project_id) {
                Ok(_) => {
                    println!("✅ Set '{}' as active project", project_name);
                }
                Err(e) => {
                    eprintln!("Failed to set active project: {}", e);
                    std::process::exit(1);
                }
            }
        }
        
        ProjectCommands::Upload { project, file } => {
            let project_id = match project {
                Some(name) => {
                    // Find project by name
                    let projects = match project_manager.list_projects() {
                        Ok(p) => p,
                        Err(e) => {
                            eprintln!("Failed to list projects: {}", e);
                            std::process::exit(1);
                        }
                    };
                    match projects.iter().find(|p| p.name.eq_ignore_ascii_case(name)) {
                        Some(proj) => proj.id.clone(),
                        None => {
                            eprintln!("Project '{}' not found", name);
                            std::process::exit(1);
                        }
                    }
                }
                None => {
                    // Use active project or default
                    match project_manager.get_active_project() {
                        Ok(Some(active_id)) => active_id,
                        Ok(None) => "default".to_string(),
                        Err(_) => "default".to_string(),
                    }
                }
            };
            
            match std::fs::read(file) {
                Ok(content) => {
                    let filename = std::path::Path::new(file)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string();
                    
                    match project_manager.upload_file(&project_id, filename, &content) {
                        Ok(file_obj) => {
                            // Update project with new file
                            if let Ok(Some(mut proj)) = project_manager.load_project(&project_id) {
                                proj.add_file(file_obj.clone());
                                if let Err(e) = project_manager.save_project(&proj) {
                                    eprintln!("Warning: Failed to save project: {}", e);
                                }
                                println!("✅ Uploaded file '{}' to project '{}'", file_obj.filename, proj.name);
                            } else {
                                println!("✅ Uploaded file '{}' to project '{}'", file_obj.filename, project_id);
                            }
                            println!("   File ID: {}", file_obj.id);
                        }
                        Err(e) => {
                            eprintln!("Failed to upload file: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to read file '{}': {}", file, e);
                    std::process::exit(1);
                }
            }
        }

        ProjectCommands::Files { project } => {
            let project_id = match project {
                Some(name) => {
                    // Find project by name
                    let projects = match project_manager.list_projects() {
                        Ok(p) => p,
                        Err(e) => {
                            eprintln!("Failed to list projects: {}", e);
                            std::process::exit(1);
                        }
                    };
                    match projects.iter().find(|p| p.name.eq_ignore_ascii_case(name)) {
                        Some(proj) => proj.id.clone(),
                        None => {
                            eprintln!("Project '{}' not found", name);
                            std::process::exit(1);
                        }
                    }
                }
                None => {
                    // Use active project or default
                    match project_manager.get_active_project() {
                        Ok(Some(active_id)) => active_id,
                        Ok(None) => "default".to_string(),
                        Err(_) => "default".to_string(),
                    }
                }
            };
            
            match project_manager.load_project(&project_id) {
                Ok(Some(proj)) => {
                    if proj.files.is_empty() {
                        println!("No files in project '{}'", proj.name);
                    } else {
                        println!("Files in project '{}':", proj.name);
                        for file in &proj.files {
                            println!("  {} - {} ({} bytes, {})", 
                                   file.id, 
                                   file.filename,
                                   file.size,
                                   file.uploaded_at.format("%Y-%m-%d %H:%M:%S"));
                        }
                    }
                }
                Ok(None) => {
                    eprintln!("Project '{}' not found", project_id);
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("Failed to load project: {}", e);
                    std::process::exit(1);
                }
            }
        }

        ProjectCommands::AddGoal { project, goal } => {
            let project_id = match project {
                Some(name) => {
                    // Find project by name
                    let projects = match project_manager.list_projects() {
                        Ok(p) => p,
                        Err(e) => {
                            eprintln!("Failed to list projects: {}", e);
                            std::process::exit(1);
                        }
                    };
                    match projects.iter().find(|p| p.name.eq_ignore_ascii_case(name)) {
                        Some(proj) => proj.id.clone(),
                        None => {
                            eprintln!("Project '{}' not found", name);
                            std::process::exit(1);
                        }
                    }
                }
                None => {
                    // Use active project or default
                    match project_manager.get_active_project() {
                        Ok(Some(active_id)) => active_id,
                        Ok(None) => "default".to_string(),
                        Err(_) => "default".to_string(),
                    }
                }
            };
            
            // Load or create project
            let mut proj = match project_manager.load_project(&project_id) {
                Ok(Some(p)) => p,
                Ok(None) if project_id == "default" => {
                    match project_manager.get_default_project() {
                        Ok(p) => p,
                        Err(e) => {
                            eprintln!("Failed to create default project: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                Ok(None) => {
                    eprintln!("Project '{}' not found", project_id);
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("Failed to load project: {}", e);
                    std::process::exit(1);
                }
            };

            let order = proj.goals.len() as u32;
            let goal_obj = Goal::new(goal.clone(), order);
            proj.add_goal(goal_obj.clone());

            match project_manager.save_project(&proj) {
                Ok(_) => {
                    println!("✅ Added goal to project '{}': {}", proj.name, goal);
                    println!("   Goal ID: {}", goal_obj.id);
                }
                Err(e) => {
                    eprintln!("Failed to save project: {}", e);
                    std::process::exit(1);
                }
            }
        }

        ProjectCommands::RemoveGoal { project, goal_id } => {
            let project_id = match project {
                Some(name) => {
                    // Find project by name
                    let projects = match project_manager.list_projects() {
                        Ok(p) => p,
                        Err(e) => {
                            eprintln!("Failed to list projects: {}", e);
                            std::process::exit(1);
                        }
                    };
                    match projects.iter().find(|p| p.name.eq_ignore_ascii_case(name)) {
                        Some(proj) => proj.id.clone(),
                        None => {
                            eprintln!("Project '{}' not found", name);
                            std::process::exit(1);
                        }
                    }
                }
                None => {
                    // Use active project or default
                    match project_manager.get_active_project() {
                        Ok(Some(active_id)) => active_id,
                        Ok(None) => "default".to_string(),
                        Err(_) => "default".to_string(),
                    }
                }
            };
            
            match project_manager.load_project(&project_id) {
                Ok(Some(mut proj)) => {
                    if proj.remove_goal(goal_id) {
                        match project_manager.save_project(&proj) {
                            Ok(_) => {
                                println!("✅ Removed goal '{}' from project '{}'", goal_id, proj.name);
                            }
                            Err(e) => {
                                eprintln!("Failed to save project: {}", e);
                                std::process::exit(1);
                            }
                        }
                    } else {
                        eprintln!("Goal '{}' not found in project '{}'", goal_id, proj.name);
                        std::process::exit(1);
                    }
                }
                Ok(None) => {
                    eprintln!("Project '{}' not found", project_id);
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("Failed to load project: {}", e);
                    std::process::exit(1);
                }
            }
        }
        
        ProjectCommands::AddContext { project, context } => {
            let project_id = match project {
                Some(name) => {
                    // Find project by name
                    let projects = match project_manager.list_projects() {
                        Ok(p) => p,
                        Err(e) => {
                            eprintln!("Failed to list projects: {}", e);
                            std::process::exit(1);
                        }
                    };
                    match projects.iter().find(|p| p.name.eq_ignore_ascii_case(name)) {
                        Some(proj) => proj.id.clone(),
                        None => {
                            eprintln!("Project '{}' not found", name);
                            std::process::exit(1);
                        }
                    }
                }
                None => {
                    // Use active project or default
                    match project_manager.get_active_project() {
                        Ok(Some(active_id)) => active_id,
                        Ok(None) => "default".to_string(),
                        Err(_) => "default".to_string(),
                    }
                }
            };
            
            // Load or create project
            let mut proj = match project_manager.load_project(&project_id) {
                Ok(Some(p)) => p,
                Ok(None) if project_id == "default" => {
                    match project_manager.get_default_project() {
                        Ok(p) => p,
                        Err(e) => {
                            eprintln!("Failed to create default project: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                Ok(None) => {
                    eprintln!("Project '{}' not found", project_id);
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("Failed to load project: {}", e);
                    std::process::exit(1);
                }
            };

            let order = proj.contexts.len() as u32;
            let context_obj = Context::new(context.clone(), order);
            proj.add_context(context_obj.clone());

            match project_manager.save_project(&proj) {
                Ok(_) => {
                    println!("✅ Added context to project '{}': {}", proj.name, context);
                    println!("   Context ID: {}", context_obj.id);
                }
                Err(e) => {
                    eprintln!("Failed to save project: {}", e);
                    std::process::exit(1);
                }
            }
        }

        ProjectCommands::RemoveContext { project, context_id } => {
            let project_id = match project {
                Some(name) => {
                    // Find project by name
                    let projects = match project_manager.list_projects() {
                        Ok(p) => p,
                        Err(e) => {
                            eprintln!("Failed to list projects: {}", e);
                            std::process::exit(1);
                        }
                    };
                    match projects.iter().find(|p| p.name.eq_ignore_ascii_case(name)) {
                        Some(proj) => proj.id.clone(),
                        None => {
                            eprintln!("Project '{}' not found", name);
                            std::process::exit(1);
                        }
                    }
                }
                None => {
                    // Use active project or default
                    match project_manager.get_active_project() {
                        Ok(Some(active_id)) => active_id,
                        Ok(None) => "default".to_string(),
                        Err(_) => "default".to_string(),
                    }
                }
            };
            
            match project_manager.load_project(&project_id) {
                Ok(Some(mut proj)) => {
                    if proj.remove_context(context_id) {
                        match project_manager.save_project(&proj) {
                            Ok(_) => {
                                println!("✅ Removed context '{}' from project '{}'", context_id, proj.name);
                            }
                            Err(e) => {
                                eprintln!("Failed to save project: {}", e);
                                std::process::exit(1);
                            }
                        }
                    } else {
                        eprintln!("Context '{}' not found in project '{}'", context_id, proj.name);
                        std::process::exit(1);
                    }
                }
                Ok(None) => {
                    eprintln!("Project '{}' not found", project_id);
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("Failed to load project: {}", e);
                    std::process::exit(1);
                }
            }
        }
        
        ProjectCommands::RemoveFile { project, file_id } => {
            let project_id = match project {
                Some(name) => {
                    // Find project by name
                    let projects = match project_manager.list_projects() {
                        Ok(p) => p,
                        Err(e) => {
                            eprintln!("Failed to list projects: {}", e);
                            std::process::exit(1);
                        }
                    };
                    match projects.iter().find(|p| p.name.eq_ignore_ascii_case(name)) {
                        Some(proj) => proj.id.clone(),
                        None => {
                            eprintln!("Project '{}' not found", name);
                            std::process::exit(1);
                        }
                    }
                }
                None => {
                    // Use active project or default
                    match project_manager.get_active_project() {
                        Ok(Some(active_id)) => active_id,
                        Ok(None) => "default".to_string(),
                        Err(_) => "default".to_string(),
                    }
                }
            };
            
            match project_manager.load_project(&project_id) {
                Ok(Some(mut proj)) => {
                    // Remove from project metadata
                    if proj.remove_file(file_id) {
                        // Also delete the actual file
                        if let Err(e) = project_manager.delete_file(&project_id, file_id) {
                            eprintln!("Warning: Failed to delete file from disk: {}", e);
                        }
                        
                        match project_manager.save_project(&proj) {
                            Ok(_) => {
                                println!("✅ Removed file '{}' from project '{}'", file_id, proj.name);
                            }
                            Err(e) => {
                                eprintln!("Failed to save project: {}", e);
                                std::process::exit(1);
                            }
                        }
                    } else {
                        eprintln!("File '{}' not found in project '{}'", file_id, proj.name);
                        std::process::exit(1);
                    }
                }
                Ok(None) => {
                    eprintln!("Project '{}' not found", project_id);
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("Failed to load project: {}", e);
                    std::process::exit(1);
                }
            }
        }
        
        ProjectCommands::Show { project } => {
            let project_id = match project {
                Some(name) => {
                    // Find project by name
                    let projects = match project_manager.list_projects() {
                        Ok(p) => p,
                        Err(e) => {
                            eprintln!("Failed to list projects: {}", e);
                            std::process::exit(1);
                        }
                    };
                    match projects.iter().find(|p| p.name.eq_ignore_ascii_case(name)) {
                        Some(proj) => proj.id.clone(),
                        None => {
                            eprintln!("Project '{}' not found", name);
                            std::process::exit(1);
                        }
                    }
                }
                None => {
                    // Use active project or default
                    match project_manager.get_active_project() {
                        Ok(Some(active_id)) => active_id,
                        Ok(None) => "default".to_string(),
                        Err(_) => "default".to_string(),
                    }
                }
            };
            
            match project_manager.load_project(&project_id) {
                Ok(Some(proj)) => {
                    println!("Project Details:");
                    println!("  Name: {}", proj.name);
                    println!("  ID: {}", proj.id);
                    println!("  Created: {}", proj.created_at.format("%Y-%m-%d %H:%M:%S"));
                    println!("  Updated: {}", proj.updated_at.format("%Y-%m-%d %H:%M:%S"));
                    
                    println!("\nGoals ({}):", proj.goals.len());
                    for goal in &proj.goals {
                        println!("  {}. {} (ID: {})", goal.order + 1, goal.text, goal.id);
                    }
                    
                    println!("\nContexts ({}):", proj.contexts.len());
                    for context in &proj.contexts {
                        println!("  {}. {} (ID: {})", context.order + 1, context.text, context.id);
                    }
                    
                    println!("\nFiles ({}):", proj.files.len());
                    for file in &proj.files {
                        println!("  {} - {} ({} bytes)", file.filename, file.id, file.size);
                    }
                }
                Ok(None) => {
                    if project_id == "default" {
                        println!("No default project exists. Creating one...");
                        match project_manager.get_default_project() {
                            Ok(proj) => {
                                println!("✅ Created default project");
                                println!("  Name: {}", proj.name);
                                println!("  ID: {}", proj.id);
                            }
                            Err(e) => {
                                eprintln!("Failed to create default project: {}", e);
                                std::process::exit(1);
                            }
                        }
                    } else {
                        eprintln!("Project '{}' not found", project_id);
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to load project: {}", e);
                    std::process::exit(1);
                }
            }
        }

        ProjectCommands::Run { project, goals, format, warnings, clipboard, no_thinking } => {
            let project_id = match project {
                Some(name) => {
                    // Find project by name
                    let projects = match project_manager.list_projects() {
                        Ok(p) => p,
                        Err(e) => {
                            eprintln!("Failed to list projects: {}", e);
                            std::process::exit(1);
                        }
                    };
                    match projects.iter().find(|p| p.name.eq_ignore_ascii_case(name)) {
                        Some(proj) => Some(proj.id.clone()),
                        None => {
                            eprintln!("Project '{}' not found", name);
                            std::process::exit(1);
                        }
                    }
                }
                None => {
                    // Use active project if available
                    project_manager.get_active_project().unwrap_or(None)
                }
            };
            
            match prompt::structure_reasoning_with_project(
                project_id.as_deref(),
                goals,
                format,
                warnings,
                *clipboard,
                None,
                *no_thinking,
            ) {
                Ok(_) => {
                    // Success
                }
                Err(e) => {
                    eprintln!("Failed to run prompt with project: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}
