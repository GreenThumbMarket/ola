// Prompt handling logic module
use serde_json::json;
use std::path::Path;
use std::fs;
use regex::Regex;

use crate::api::{create_api_client_from_config, format_prompt};
use crate::utils::{clipboard, output, piping};

// Structure to hold the result of a prompt for testing
#[derive(Debug)]
pub struct PromptResult {
    pub content: String,
    pub model: String,
}

/// Main function for structured reasoning with <think> blocks
pub fn structure_reasoning(
    goals: &str,
    return_type: &str,
    warnings: &str,
    clipboard: bool,
    context: Option<&str>,
    no_thinking: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Try to load settings
    let settings = crate::settings::Settings::load().unwrap_or_default();
    
    // Format the prompt with goals, return type, warnings, and optional context
    let mut input_data = format_prompt(goals, return_type, warnings, context);
    
    // Read and append hints if available
    append_hints_if_available(&mut input_data)?;
    
    // Load current configuration and create API client
    let api_client = create_api_client_from_config()?;
    
    // Use model from config, settings, or fallback to default
    let config = crate::config::Config::load()?;
    let provider_config = config.get_active_provider().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No active provider configured. Run 'ola configure' first.",
        )
    })?;
    
    let model = provider_config
        .model
        .as_deref()
        .unwrap_or(&settings.default_model);
    println!("Using model: {}", model);
    
    // Stream the response
    let response = stream_response(&api_client, &input_data, model, no_thinking)?;
    
    // Handle clipboard copy if requested
    if clipboard {
        match clipboard::copy_to_clipboard(&response) {
            Ok(_) => output::print_success("Response copied to clipboard"),
            Err(e) => output::print_error(&format!("Failed to copy to clipboard: {}", e))
        }
    }
    
    // Log session if enabled in settings
    if settings.behavior.enable_logging {
        log_session(goals, return_type, warnings, model, &response)?;
    }
    
    Ok(())
}

/// Stream raw prompt without structured reasoning
pub fn stream_non_think(
    prompt: &str,
    clipboard: bool,
    context: Option<&str>,
    filter_thinking: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Try to load settings
    let settings = crate::settings::Settings::load().unwrap_or_default();
    
    // Format the prompt with optional context
    let mut input_data = if let Some(ctx) = context {
        format!("{}\nContext: {}", prompt, ctx)
    } else {
        prompt.to_string()
    };
    
    // Read and append hints if available
    append_hints_if_available(&mut input_data)?;
    
    // Create API client
    let api_client = create_api_client_from_config()?;
    
    // Get model information
    let config = crate::config::Config::load()?;
    let provider_config = config.get_active_provider().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No active provider configured. Run 'ola configure' first.",
        )
    })?;
    
    let model = provider_config
        .model
        .as_deref()
        .unwrap_or(&settings.default_model);
    println!("Using model: {}", model);
    
    // Stream the response
    let response = stream_response(&api_client, &input_data, model, filter_thinking)?;
    
    // Handle clipboard copy if requested
    if clipboard {
        match clipboard::copy_to_clipboard(&response) {
            Ok(_) => output::print_success("Response copied to clipboard"),
            Err(e) => output::print_error(&format!("Failed to copy to clipboard: {}", e))
        }
    }
    
    // Log session if enabled in settings
    if settings.behavior.enable_logging {
        let log_entry = json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "prompt": prompt,
            "model": model,
            "output_length": response.len(),
        });
        
        if let Err(e) = piping::append_to_log(&settings.behavior.log_file, &log_entry.to_string()) {
            eprintln!("Failed to log session: {}", e);
        }
    }
    
    Ok(())
}

// Helper function to stream response with thinking block filtering if needed
fn stream_response(
    api_client: &crate::api::ApiClient,
    prompt: &str,
    model: &str,
    filter_thinking: bool
) -> Result<String, Box<dyn std::error::Error>> {
    // Get the raw response
    let response = api_client.stream_prompt(prompt, model)?;
    
    // If we need to filter thinking blocks, process the response
    if filter_thinking {
        // Use regex to remove thinking blocks
        let re = Regex::new(r"<think>.*?</think>")?;
        let filtered_response = re.replace_all(&response, "").to_string();
        Ok(filtered_response)
    } else {
        Ok(response)
    }
}

// Helper function to read and append hints from .olaHints file
fn append_hints_if_available(input_data: &mut String) -> Result<(), Box<dyn std::error::Error>> {
    // Try to read hints from a local .olaHints file; if not found, fallback to global hints
    let mut hints = String::new();
    
    // Check local file .olaHints in the current directory
    if Path::new("./.olaHints").exists() {
        hints = fs::read_to_string("./.olaHints")?;
    } else {
        // Fallback to global hints in ~/.ola-hints/olaHints
        if let Ok(home) = std::env::var("HOME") {
            let global_path = format!("{}/.ola-hints/olaHints", home);
            if Path::new(&global_path).exists() {
                hints = fs::read_to_string(global_path)?;
            }
        }
    }

    // If hints were found, append them to the input data
    if !hints.is_empty() {
        input_data.push_str(&format!("\nHINTS: {}", hints));
    }
    
    Ok(())
}

// Helper function to log session information
fn log_session(
    goals: &str,
    return_type: &str,
    warnings: &str,
    model: &str,
    response: &str
) -> Result<(), Box<dyn std::error::Error>> {
    let settings = crate::settings::Settings::load().unwrap_or_default();
    
    // Get recursion wave number if present
    let wave_number = std::env::var("OLA_RECURSION_WAVE")
        .ok()
        .and_then(|s| s.parse::<u8>().ok());
    
    // Build log entry with optional recursion information
    let mut log_entry = json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "goals": goals,
        "return_format": return_type,
        "warnings": warnings,
        "model": model,
        "output_length": response.len(),
    });
    
    // Add recursion wave info if available
    if let Some(wave) = wave_number {
        log_entry["recursion_wave"] = json!(wave);
    }
    
    piping::append_to_log(&settings.behavior.log_file, &log_entry.to_string())?;
    Ok(())
}

/// Interactive iterations with user feedback for LLM responses  
pub fn interactive_iterations(
    goals: &str,
    return_type: &str,
    warnings: &str,
    clipboard: bool,
    context: Option<&str>,
    no_thinking: bool,
    max_iterations: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut conversation_history = Vec::new();
    
    for iteration in 1..=max_iterations {
        println!("\n🔄 Iteration {}/{}", iteration, max_iterations);
        println!("{}", "─".repeat(50));
        
        // Execute the current prompt
        let response = execute_feedback_prompt(
            goals,
            return_type,
            warnings,
            context,
            no_thinking,
            &conversation_history,
        )?;
        
        // Store this interaction
        conversation_history.push(FeedbackInteraction {
            iteration: iteration as usize,
            goals: goals.to_string(),
            response: response.clone(),
        });
        
        // Handle clipboard copy if requested (only for final iteration)
        if clipboard && iteration == max_iterations {
            match crate::utils::clipboard::copy_to_clipboard(&response) {
                Ok(_) => eprintln!("✅ Final response copied to clipboard"),
                Err(e) => eprintln!("❌ Failed to copy to clipboard: {}", e)
            }
        }
        
        // Prompt for user feedback for next iteration (except last)
        if iteration < max_iterations {
            println!("\n💭 How would you like to improve this response for iteration {}?", iteration + 1);
            println!("Examples of helpful feedback:");
            println!("  • Make it more detailed/concise");
            println!("  • Focus on specific aspects");
            println!("  • Add examples or clarifications");
            println!("  • Change the tone or approach");
            println!("  • Address missing points");
            println!();
            print!("Your feedback (or press Enter for general improvement): ");
            std::io::Write::flush(&mut std::io::stdout())?;
            
            let mut user_feedback = String::new();
            std::io::stdin().read_line(&mut user_feedback)?;
            let user_feedback = user_feedback.trim();
            
            if !user_feedback.is_empty() {
                conversation_history.push(FeedbackInteraction {
                    iteration: iteration as usize,
                    goals: format!("FEEDBACK: {}", user_feedback),
                    response: String::new(),
                });
                println!("✅ Feedback recorded: {}", user_feedback);
            } else {
                // If no feedback provided, use a more specific improvement prompt
                let default_feedback = format!(
                    "Please review your previous response and improve it by making it more comprehensive, clearer, and better structured. This is iteration {} of {}, so focus on refinement and quality enhancement.", 
                    iteration + 1, max_iterations
                );
                
                conversation_history.push(FeedbackInteraction {
                    iteration: iteration as usize,
                    goals: format!("FEEDBACK: {}", default_feedback),
                    response: String::new(),
                });
                println!("✅ Using default improvement guidance for next iteration");
            }
        }
    }
    
    // Provide feedback analysis summary
    let feedback_summary = analyze_feedback_patterns(&conversation_history);
    if !feedback_summary.is_empty() {
        println!("\n📊 Feedback Analysis Summary:");
        println!("{}", feedback_summary);
    }
    
    println!("\n✅ Completed {} iterations with feedback", max_iterations);
    Ok(())
}

/// Interactive feedback loop for iterating on LLM responses
pub fn interactive_feedback(
    goals: &str,
    return_type: &str,
    warnings: &str,
    clipboard: bool,
    context: Option<&str>,
    no_thinking: bool,
    max_iterations: Option<u8>,
) -> Result<(), Box<dyn std::error::Error>> {
    use dialoguer::{theme::ColorfulTheme, Input, Select};
    
    let mut conversation_history = Vec::new();
    let mut iteration = 1;
    
    // Initial prompt
    let mut current_goals = goals.to_string();
    let mut current_context = context.map(|c| c.to_string());
    
    loop {
        println!("\n🔄 Iteration {}", iteration);
        println!("{}", "─".repeat(50));
        
        // Execute the current prompt
        let response = execute_feedback_prompt(
            &current_goals,
            return_type,
            warnings,
            current_context.as_deref(),
            no_thinking,
            &conversation_history,
        )?;
        
        // Store this interaction
        conversation_history.push(FeedbackInteraction {
            iteration,
            goals: current_goals.clone(),
            response: response.clone(),
        });
        
        // Handle clipboard copy if requested
        if clipboard {
            match clipboard::copy_to_clipboard(&response) {
                Ok(_) => output::print_success("Response copied to clipboard"),
                Err(e) => output::print_error(&format!("Failed to copy to clipboard: {}", e))
            }
        }
        
        // Check if we should auto-iterate or ask user for next action
        if let Some(max_iter) = max_iterations {
            if iteration < max_iter.into() {
                // Ask user if they want to continue to next iteration
                println!("\n🤔 Continue to iteration {} of {}?", iteration + 1, max_iter);
                let continue_options = vec![
                    "Yes, continue with automatic improvement",
                    "Yes, but let me provide specific feedback", 
                    "No, finish here"
                ];
                
                let continue_choice = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("What would you like to do?")
                    .items(&continue_options)
                    .default(0)
                    .interact()?;
                
                match continue_choice {
                    0 => {
                        // Auto-iterate with generic improvement feedback
                        let auto_feedback = format!("Please improve this response. Make it more detailed, accurate, and helpful. This is iteration {} of {}.", 
                            iteration + 1, max_iter);
                        
                        println!("\n🔄 Continuing to iteration {} of {}...", iteration + 1, max_iter);
                        
                        // Add auto-feedback to conversation history
                        conversation_history.push(FeedbackInteraction {
                            iteration,
                            goals: format!("FEEDBACK: {}", auto_feedback),
                            response: String::new(),
                        });
                        iteration += 1;
                        continue; // Skip the interactive menu
                    },
                    1 => {
                        // Let user provide specific feedback
                        let user_feedback: String = Input::with_theme(&ColorfulTheme::default())
                            .with_prompt("💬 Your feedback for the next iteration")
                            .interact_text()?;
                        
                        if !user_feedback.trim().is_empty() {
                            conversation_history.push(FeedbackInteraction {
                                iteration,
                                goals: format!("FEEDBACK: {}", user_feedback),
                                response: String::new(),
                            });
                            iteration += 1;
                            continue;
                        }
                    },
                    2 => {
                        // Finish early
                        println!("\n✅ Completed {} iterations (stopped early)", iteration);
                        break;
                    },
                    _ => unreachable!(),
                }
            } else {
                // Reached max iterations
                println!("\n✅ Completed {} iterations", max_iter);
                break;
            }
        }
        
        // Interactive mode - ask user for next action
        println!("\n🤔 What would you like to do next?");
        let options = vec![
            "Provide feedback and iterate",
            "Add more context",
            "Change goals",
            "Finish (exit feedback loop)",
        ];
        
        let choice = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Choose an action")
            .items(&options)
            .default(0)
            .interact()?;
        
        match choice {
            0 => {
                // Provide feedback and iterate
                let feedback: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("💬 Your feedback")
                    .interact_text()?;
                
                if !feedback.trim().is_empty() {
                    // Add feedback to conversation history
                    conversation_history.push(FeedbackInteraction {
                        iteration,
                        goals: format!("FEEDBACK: {}", feedback),
                        response: String::new(),
                    });
                    iteration += 1;
                }
            },
            1 => {
                // Add more context
                let additional_context: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("📝 Additional context")
                    .interact_text()?;
                
                if !additional_context.trim().is_empty() {
                    current_context = match current_context {
                        Some(existing) => Some(format!("{}\n\nAdditional Context: {}", existing, additional_context)),
                        None => Some(additional_context),
                    };
                    iteration += 1;
                }
            },
            2 => {
                // Change goals
                let new_goals: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("🎯 New goals")
                    .default(current_goals.clone())
                    .interact_text()?;
                
                if new_goals != current_goals {
                    current_goals = new_goals;
                    iteration += 1;
                }
            },
            3 => {
                // Finish
                println!("\n✅ Feedback session completed after {} iterations", iteration);
                break;
            },
            _ => unreachable!(),
        }
    }
    
    Ok(())
}

// Structure to hold feedback interactions
#[derive(Debug, Clone)]
struct FeedbackInteraction {
    iteration: usize,
    goals: String,
    response: String,
}

// Helper function to execute a prompt with feedback history
fn execute_feedback_prompt(
    goals: &str,
    return_type: &str,
    warnings: &str,
    context: Option<&str>,
    no_thinking: bool,
    conversation_history: &[FeedbackInteraction],
) -> Result<String, Box<dyn std::error::Error>> {
    let settings = crate::settings::Settings::load().unwrap_or_default();
    
    // Build the prompt with conversation history
    let mut full_prompt = format_prompt(goals, return_type, warnings, context);
    
    // Add conversation history if present
    if !conversation_history.is_empty() {
        full_prompt.push_str("\n\n--- ITERATION HISTORY ---\n");
        full_prompt.push_str("The following shows the conversation history from previous iterations.\n");
        full_prompt.push_str("Pay close attention to the user feedback and incorporate it into your next response.\n\n");
        
        let mut current_iteration = 0;
        for interaction in conversation_history {
            if interaction.goals.starts_with("FEEDBACK: ") {
                full_prompt.push_str(&format!("📝 User Feedback for Iteration {}: {}\n\n", 
                    current_iteration + 1,
                    interaction.goals.strip_prefix("FEEDBACK: ").unwrap_or(&interaction.goals)));
            } else if !interaction.response.is_empty() {
                current_iteration = interaction.iteration;
                full_prompt.push_str(&format!("🤖 Previous Response (Iteration {}): {}\n\n", 
                    interaction.iteration, interaction.response));
            }
        }
        
        // Add specific instruction for incorporating feedback
        full_prompt.push_str("--- FEEDBACK INCORPORATION INSTRUCTIONS ---\n");
        full_prompt.push_str("Please carefully review the above feedback and previous responses.\n");
        full_prompt.push_str("Incorporate the user's feedback to improve your response.\n");
        full_prompt.push_str("Address any specific concerns or suggestions mentioned in the feedback.\n");
        full_prompt.push_str("Build upon the previous work while making the requested improvements.\n");
        full_prompt.push_str("--- END HISTORY ---\n\n");
    }
    
    // Read and append hints if available
    append_hints_if_available(&mut full_prompt)?;
    
    // Load configuration and create API client
    let api_client = create_api_client_from_config()?;
    let config = crate::config::Config::load()?;
    let provider_config = config.get_active_provider().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No active provider configured. Run 'ola configure' first.",
        )
    })?;
    
    let model = provider_config
        .model
        .as_deref()
        .unwrap_or(&settings.default_model);
    
    // Stream the response
    let response = stream_response(&api_client, &full_prompt, model, no_thinking)?;
    
    // Log session if enabled in settings
    if settings.behavior.enable_logging {
        log_feedback_session(goals, return_type, warnings, model, &response, conversation_history.len())?;
    }
    
    Ok(response)
}

// Helper function to analyze feedback patterns and provide insights
fn analyze_feedback_patterns(conversation_history: &[FeedbackInteraction]) -> String {
    let feedback_interactions: Vec<_> = conversation_history
        .iter()
        .filter(|interaction| interaction.goals.starts_with("FEEDBACK: "))
        .collect();
    
    if feedback_interactions.is_empty() {
        return String::new();
    }
    
    let mut analysis = Vec::new();
    
    // Count user-provided vs default feedback
    let user_feedback_count = feedback_interactions
        .iter()
        .filter(|interaction| {
            let feedback = interaction.goals.strip_prefix("FEEDBACK: ").unwrap_or("");
            !feedback.contains("Please review your previous response and improve it")
        })
        .count();
    
    let default_feedback_count = feedback_interactions.len() - user_feedback_count;
    
    analysis.push(format!("  • {} iterations with custom user feedback", user_feedback_count));
    if default_feedback_count > 0 {
        analysis.push(format!("  • {} iterations with default improvement guidance", default_feedback_count));
    }
    
    // Analyze feedback themes
    let mut themes = Vec::new();
    let feedback_text: String = feedback_interactions
        .iter()
        .map(|interaction| interaction.goals.strip_prefix("FEEDBACK: ").unwrap_or(""))
        .collect::<Vec<_>>()
        .join(" ");
    
    let feedback_lower = feedback_text.to_lowercase();
    
    if feedback_lower.contains("detail") || feedback_lower.contains("comprehensive") || feedback_lower.contains("thorough") {
        themes.push("Detail enhancement");
    }
    if feedback_lower.contains("concise") || feedback_lower.contains("brief") || feedback_lower.contains("shorter") {
        themes.push("Brevity improvement");
    }
    if feedback_lower.contains("example") || feedback_lower.contains("sample") {
        themes.push("Example requests");
    }
    if feedback_lower.contains("clarif") || feedback_lower.contains("explain") || feedback_lower.contains("clear") {
        themes.push("Clarity improvements");
    }
    if feedback_lower.contains("tone") || feedback_lower.contains("style") {
        themes.push("Tone/style adjustments");
    }
    if feedback_lower.contains("focus") || feedback_lower.contains("specific") {
        themes.push("Focus refinement");
    }
    
    if !themes.is_empty() {
        analysis.push(format!("  • Common feedback themes: {}", themes.join(", ")));
    }
    
    // Provide recommendations
    if user_feedback_count < feedback_interactions.len() / 2 {
        analysis.push("  💡 Tip: Providing specific feedback helps tailor responses to your needs".to_string());
    }
    
    analysis.join("\n")
}

// Helper function to log feedback session information
fn log_feedback_session(
    goals: &str,
    return_type: &str,
    warnings: &str,
    model: &str,
    response: &str,
    iteration: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let settings = crate::settings::Settings::load().unwrap_or_default();
    
    let log_entry = json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "session_type": "feedback",
        "iteration": iteration,
        "goals": goals,
        "return_format": return_type,
        "warnings": warnings,
        "model": model,
        "output_length": response.len(),
    });
    
    piping::append_to_log(&settings.behavior.log_file, &log_entry.to_string())?;
    Ok(())
}

// Testing functions
// Test version of structure_reasoning
pub fn structure_reasoning_test(
    goals: &str,
    return_type: &str,
    warnings: &str,
    clipboard: bool,
    context: Option<&str>,
    no_thinking: bool,
) -> Result<PromptResult, Box<dyn std::error::Error>> {
    // This is a testable version that returns the response instead of printing it
    let settings = crate::settings::Settings::load().unwrap_or_default();
    
    // Build the input data with optional context
    let input_data = if let Some(ctx) = context {
        format!(
            "Goals: {}\nReturn Format: {}\nWarnings: {}\nContext: {}",
            goals, return_type, warnings, ctx
        )
    } else {
        format!(
            "Goals: {}\nReturn Format: {}\nWarnings: {}",
            goals, return_type, warnings
        )
    };

    // Load configuration
    let config = crate::config::Config::load()?;
    let provider_config = config.get_active_provider().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No active provider configured.",
        )
    })?;

    // Use model from config or settings
    let model = provider_config
        .model
        .as_deref()
        .unwrap_or(&settings.default_model);
    
    // Here's where we would mock the API call in tests
    // For testing, we'll just return the input as the response
    Ok(PromptResult {
        content: format!("This is a mocked response from the {} API.", provider_config.provider),
        model: model.to_string(),
    })
}

// Test version of stream_non_think
pub fn stream_non_think_test(
    prompt: &str,
    clipboard: bool,
    context: Option<&str>,
    filter_thinking: bool,
) -> Result<PromptResult, Box<dyn std::error::Error>> {
    // This is a testable version that returns the response instead of printing it
    let settings = crate::settings::Settings::load().unwrap_or_default();
    
    // Build the input data with optional context
    let input_data = if let Some(ctx) = context {
        format!("{}\nContext: {}", prompt, ctx)
    } else {
        prompt.to_string()
    };

    // Load configuration
    let config = crate::config::Config::load()?;
    let provider_config = config.get_active_provider().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No active provider configured.",
        )
    })?;

    // Use model from config or settings
    let model = provider_config
        .model
        .as_deref()
        .unwrap_or(&settings.default_model);
    
    // Here's where we would mock the API call in tests
    // For testing, we'll just return the input as the response
    Ok(PromptResult {
        content: format!("This is a mocked response from the {} API.", provider_config.provider),
        model: model.to_string(),
    })
}