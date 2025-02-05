use clap::{Parser, Subcommand};
use dialoguer::{theme::ColorfulTheme, Input, Select};
use std::fs::OpenOptions;
use std::io::Write;
use chrono::Utc;
use serde_json::json;

mod prompt;

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
    Configure { /// Configure the provider for reasoning models
        #[arg(short, long)]
        provider: String, /// e.g., openai, anthropic
        /// Optional configuration details
        #[arg(short, long)]
        details: Option<String>,
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
        Some(Commands::Configure { provider, details }) => {
            println!("Configuring provider: {}", provider);
            if let Some(info) = details {
                println!("Additional details: {}", info);
            } else {
                println!("No additional details provided.");
            }
            // Here you could store configuration details in a file or environment as needed
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
        .with_prompt("Goals: ")
        .default("Anonymous".into())
        .interact_text()
        .unwrap();

    // Ask user for their return format
    let return_format: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Return Format: ")
        .default("Anonymous".into())
        .interact_text()
        .unwrap();

    // Ask user for their warnings
    let warnings: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Goals: ")
        .default("Anonymous".into())
        .interact_text()
        .unwrap();

    // Call the prompt function from the ola crate
    let output = prompt::structure_reasoning(&goals, &return_format, &warnings);

    // Ask user to select from some choices
    let options = vec!["Option A", "Option B", "Option C"];
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose an option")
        .items(&options)
        .default(0)
        .interact()
        .unwrap();

    println!("Selected: {}", options[selection]);
    println!("Goals: {}\nReturn Format: {}\nWarnings: {}", goals, return_format, warnings);
    println!("Prompt output: {}", output);
}

fn append_to_log(filename: &str, entry: &str) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(filename)?;
    writeln!(file, "{}", entry)?;
    Ok(())
}
