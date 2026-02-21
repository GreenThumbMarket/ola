/// Application state and logic for the TUI

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ola_core::{api::ApiClient, Config};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Main,
    ProviderSelection,
    Settings,
    Help,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Normal,
    Input,
    Processing,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct Message {
    pub content: String,
    pub is_user: bool,
    pub timestamp: Instant,
}

pub struct App {
    pub mode: AppMode,
    pub screen: Screen,
    pub should_quit: bool,

    // Chat state
    pub messages: Vec<Message>,
    pub input: String,
    pub input_cursor: usize,
    pub scroll_offset: usize,

    // Configuration
    pub config: Option<Config>,
    pub providers: Vec<String>,
    pub selected_provider_index: usize,
    pub selected_model: String,

    // Animation state
    pub animation_frame: usize,
    pub last_tick: Instant,
    pub wave_animation: bool,

    // Processing state
    pub is_thinking: bool,
    pub thinking_dots: usize,

    // UI state
    pub show_help: bool,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Result<Self> {
        let config = Config::load().ok();
        let providers = vec![
            "OpenAI".to_string(),
            "Anthropic".to_string(),
            "Gemini".to_string(),
            "Ollama".to_string(),
        ];

        let selected_model = config
            .as_ref()
            .and_then(|c| c.get_active_provider())
            .and_then(|p| p.model.clone())
            .unwrap_or_else(|| "gpt-4".to_string());

        Ok(Self {
            mode: AppMode::Normal,
            screen: Screen::Main,
            should_quit: false,
            messages: Vec::new(),
            input: String::new(),
            input_cursor: 0,
            scroll_offset: 0,
            config,
            providers,
            selected_provider_index: 0,
            selected_model,
            animation_frame: 0,
            last_tick: Instant::now(),
            wave_animation: true,
            is_thinking: false,
            thinking_dots: 0,
            show_help: false,
            status_message: None,
        })
    }

    pub async fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        match self.mode {
            AppMode::Normal => self.handle_normal_mode_key(key).await,
            AppMode::Input => self.handle_input_mode_key(key).await,
            AppMode::Processing => Ok(true), // Ignore keys during processing
            AppMode::Error(_) => {
                self.mode = AppMode::Normal;
                Ok(true)
            }
        }
    }

    async fn handle_normal_mode_key(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                if self.screen == Screen::Main {
                    self.should_quit = true;
                    Ok(false)
                } else {
                    self.screen = Screen::Main;
                    Ok(true)
                }
            }
            KeyCode::Char('i') => {
                self.mode = AppMode::Input;
                Ok(true)
            }
            KeyCode::Char('p') => {
                self.screen = Screen::ProviderSelection;
                Ok(true)
            }
            KeyCode::Char('s') => {
                self.screen = Screen::Settings;
                Ok(true)
            }
            KeyCode::Char('h') | KeyCode::Char('?') => {
                self.screen = Screen::Help;
                Ok(true)
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.messages.clear();
                self.status_message = Some("Chat cleared".to_string());
                Ok(true)
            }
            KeyCode::Up => {
                if self.scroll_offset > 0 {
                    self.scroll_offset -= 1;
                }
                Ok(true)
            }
            KeyCode::Down => {
                if self.scroll_offset < self.messages.len().saturating_sub(1) {
                    self.scroll_offset += 1;
                }
                Ok(true)
            }
            KeyCode::PageUp => {
                self.scroll_offset = self.scroll_offset.saturating_sub(10);
                Ok(true)
            }
            KeyCode::PageDown => {
                self.scroll_offset = (self.scroll_offset + 10).min(
                    self.messages.len().saturating_sub(1)
                );
                Ok(true)
            }
            _ => Ok(true),
        }
    }

    async fn handle_input_mode_key(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc => {
                self.mode = AppMode::Normal;
                Ok(true)
            }
            KeyCode::Enter => {
                if !self.input.trim().is_empty() {
                    self.send_message().await?;
                }
                Ok(true)
            }
            KeyCode::Char(c) => {
                self.input.insert(self.input_cursor, c);
                self.input_cursor += 1;
                Ok(true)
            }
            KeyCode::Backspace => {
                if self.input_cursor > 0 {
                    self.input.remove(self.input_cursor - 1);
                    self.input_cursor -= 1;
                }
                Ok(true)
            }
            KeyCode::Delete => {
                if self.input_cursor < self.input.len() {
                    self.input.remove(self.input_cursor);
                }
                Ok(true)
            }
            KeyCode::Left => {
                if self.input_cursor > 0 {
                    self.input_cursor -= 1;
                }
                Ok(true)
            }
            KeyCode::Right => {
                if self.input_cursor < self.input.len() {
                    self.input_cursor += 1;
                }
                Ok(true)
            }
            KeyCode::Home => {
                self.input_cursor = 0;
                Ok(true)
            }
            KeyCode::End => {
                self.input_cursor = self.input.len();
                Ok(true)
            }
            _ => Ok(true),
        }
    }

    async fn send_message(&mut self) -> Result<()> {
        let user_message = self.input.clone();
        self.input.clear();
        self.input_cursor = 0;

        // Add user message
        self.messages.push(Message {
            content: user_message.clone(),
            is_user: true,
            timestamp: Instant::now(),
        });

        // Switch to processing mode
        self.mode = AppMode::Processing;
        self.is_thinking = true;

        // Get API response
        match self.get_ai_response(&user_message).await {
            Ok(response) => {
                self.messages.push(Message {
                    content: response,
                    is_user: false,
                    timestamp: Instant::now(),
                });
                self.mode = AppMode::Normal;
            }
            Err(e) => {
                self.mode = AppMode::Error(format!("Error: {}", e));
            }
        }

        self.is_thinking = false;
        self.scroll_offset = self.messages.len().saturating_sub(1);
        Ok(())
    }

    async fn get_ai_response(&self, prompt: &str) -> Result<String> {
        let config = self.config.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No configuration found"))?;

        let provider_config = config.get_active_provider()
            .ok_or_else(|| anyhow::anyhow!("No active provider configured"))?;

        let provider = provider_config.provider.clone();
        let api_key = provider_config.api_key.clone();
        let model = provider_config.model.clone()
            .unwrap_or_else(|| "gpt-4".to_string());

        // Format the prompt with Goals/Format/Warnings structure
        let formatted_prompt = ola_core::api::format_prompt(
            prompt,
            "text",
            "",
            None,
        );

        // Use spawn_blocking for the synchronous API call
        let response = tokio::task::spawn_blocking(move || -> Result<String> {
            let client = ApiClient::new(&provider, &api_key, None)
                .map_err(|e| anyhow::anyhow!("Failed to create API client: {}", e))?;
            client.stream_prompt(&formatted_prompt, &model)
                .map_err(|e| anyhow::anyhow!("Failed to get AI response: {}", e))
        }).await??;

        Ok(response)
    }

    pub fn on_tick(&mut self) {
        // Update animations
        if self.last_tick.elapsed() >= Duration::from_millis(100) {
            self.animation_frame = (self.animation_frame + 1) % 12;
            if self.is_thinking {
                self.thinking_dots = (self.thinking_dots + 1) % 4;
            }
            self.last_tick = Instant::now();
        }

        // Clear status message after 3 seconds
        if let Some(_) = &self.status_message {
            if self.last_tick.elapsed() >= Duration::from_secs(3) {
                self.status_message = None;
            }
        }
    }
}