/* 
.olaHints content:
- This file contains hints for optimizing model calls.
- Use the format [Goals, Return Format, Warnings] when configuring sessions.
- Providers and session storage are easily configurable.
*/

use clap::{Parser, Subcommand};
use dialoguer::{theme::ColorfulTheme, Input, Select};
use std::fs::OpenOptions;
use std::io::Write;
use chrono::Utc;
use serde_json::json;

mod prompt;
mod config;

#[derive(Parser)]
#[command(name = "ola")]
#[command(version = "0.2.0")]
#[command(about = "A friendly CLI for prompting and optimizing reasoning model calls", long_about = None)]
struct OlaCli {
    /// Specify a subcommand
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Starts the application with optional arguments
    Start {
        /// Optional parameter for demonstration
        #[arg(short, long)]
        verbose: bool,
    },
    
    Prompt, /// Demonstrates a friendly user prompt via dialoguer
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
    /// Run a session with specified goals, return format, and warnings.
    Session {
        /// Goals for the reasoning call
        #[arg(short, long, default_value="")]
        goals: String,
        /// Expected return format
        #[arg(short = 'f', long)]
        return_format: String,
        /// Any warnings to consider
        #[arg(short, long, default_value = "")]
        warnings: String,
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
        Some(Commands::Prompt) => {
            run_prompt();
        }
        Some(Commands::Configure { provider, api_key, model }) => {
            // If no arguments provided, run interactive configuration
            if provider.is_none() && api_key.is_none() && model.is_none() {
                if let Err(e) = config::run_interactive_config() {
                    eprintln!("Configuration failed: {}", e);
                    std::process::exit(1);
                }
                return;
            }

            // Handle non-interactive configuration
            let mut config = config::Config::load().unwrap_or_else(|e| {
                eprintln!("Failed to load config: {}", e);
                std::process::exit(1);
            });

            let provider_config = config::ProviderConfig {
                provider: provider.clone().unwrap_or_else(|| {
                    eprintln!("Provider must be specified in non-interactive mode");
                    std::process::exit(1);
                }),
                api_key: api_key.clone().unwrap_or_else(|| {
                    eprintln!("API key must be specified in non-interactive mode");
                    std::process::exit(1);
                }),
                model: model.clone(),
                additional_settings: None,
            };

            // Validate the configuration
            if let Err(e) = config::validate_provider_config(&provider_config) {
                eprintln!("Invalid configuration: {}", e);
                std::process::exit(1);
            }

            config.add_provider(provider_config.clone());
            if let Err(e) = config.save() {
                eprintln!("Failed to save configuration: {}", e);
                std::process::exit(1);
            }

            println!("✅ Configuration saved for provider: {}", provider_config.provider);
            if let Some(model) = provider_config.model {
                println!("Using model: {}", model);
            }
        }
        Some(Commands::Session { goals, return_format, warnings }) => {
            println!("Running session with the following parameters:");
            println!("Goals: {}", goals);
            println!("Return Format: {}", return_format);
            if !warnings.is_empty() {
                println!("Warnings: {}", warnings);
            }
            // Simulate session processing: in a real app, you'd call the reasoning model
            let output = format!("Processed session for goals: {}", goals);
            println!("Output: {}", output);

            // Log session output to a jsonl file
            let log_entry = json!({
                "timestamp": Utc::now().to_rfc3339(),
                "goals": goals,
                "return_format": return_format,
                "warnings": warnings,
                "output": output,
            });
            if let Err(e) = append_to_log("sessions.jsonl", &log_entry.to_string()) {
                eprintln!("Failed to log session: {}", e);
            } else {
                println!("Session output logged to sessions.jsonl");
            }
        }
        None => {
            println!("No subcommand was used. Try `ola --help` for more info.");
        }
    }
}

fn run_prompt() {
    println!("Welcome to the Ola CLI Prompt!");
    
    // Ask user for their goals
    let goals: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("🏆 Goals: ")
        .default("Anonymous".into())
        .interact_text()
        .unwrap();
    
    // Ask user for their requested format
    let return_format_options = vec!["text", "json", "markdown"];
    let selected_index = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("📝 Requested Format")
        .items(&return_format_options)
        .default(0)
        .interact()
        .unwrap();
    let return_format = return_format_options[selected_index].to_string();
    
    // Ask user for their warnings
    let warnings: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("⚠️ Warnings: ")
        .default("Anonymous".into())
        .interact_text()
        .unwrap();
    
    // Call the prompt function from the ola crate
    let output = prompt::structure_reasoning(&goals, &return_format, &warnings);
    
    println!("Goals: {}\nReturn Format: {}\nWarnings: {}", goals, return_format, warnings);
    match output {
        Ok(()) => println!("Prompt executed successfully"),
        Err(e) => println!("Prompt returned error: {:?}", e),
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
