use anyhow::Result;

/// Represents the different screens/views in the TUI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Home,
    Prompt,
    Configure,
    Projects,
    Settings,
    Models,
    Help,
}

/// Input mode for text fields
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Editing,
}

/// Represents a form field with label and value
#[derive(Debug, Clone)]
pub struct FormField {
    pub label: String,
    pub value: String,
    pub placeholder: String,
}

impl FormField {
    pub fn new(label: impl Into<String>, placeholder: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: String::new(),
            placeholder: placeholder.into(),
        }
    }
}

/// Main application state
pub struct App {
    pub should_quit: bool,
    pub current_screen: Screen,
    pub input_mode: InputMode,
    pub menu_items: Vec<&'static str>,
    pub selected_menu_item: usize,

    // Prompt form fields
    pub prompt_fields: Vec<FormField>,
    pub selected_field: usize,

    // Configuration fields
    pub config_fields: Vec<FormField>,
    pub selected_config_field: usize,
    pub providers: Vec<String>,
    pub selected_provider: usize,

    // Projects
    pub projects: Vec<String>,
    pub selected_project: usize,

    // Settings
    pub settings_items: Vec<String>,
    pub selected_setting: usize,

    // Status messages
    pub status_message: Option<String>,
    pub error_message: Option<String>,

    // Output display
    pub output_text: String,
    pub output_scroll: u16,
}

impl Default for App {
    fn default() -> Self {
        Self {
            should_quit: false,
            current_screen: Screen::Home,
            input_mode: InputMode::Normal,
            menu_items: vec!["Prompt", "Configure", "Projects", "Models", "Settings", "Help", "Quit"],
            selected_menu_item: 0,

            // Initialize prompt form
            prompt_fields: vec![
                FormField::new("Goals", "What do you want to achieve?"),
                FormField::new("Format", "text"),
                FormField::new("Warnings", "Any specific warnings or constraints"),
            ],
            selected_field: 0,

            // Initialize config form
            config_fields: vec![
                FormField::new("API Key", "Enter your API key"),
                FormField::new("Model", "Select or enter model name"),
            ],
            selected_config_field: 0,
            providers: vec![
                "OpenAI".to_string(),
                "Anthropic".to_string(),
                "Ollama".to_string(),
                "Gemini".to_string(),
            ],
            selected_provider: 0,

            // Initialize projects
            projects: Vec::new(),
            selected_project: 0,

            // Initialize settings
            settings_items: vec![
                "Default Model".to_string(),
                "Default Format".to_string(),
                "Enable Logging".to_string(),
                "Log File Location".to_string(),
            ],
            selected_setting: 0,

            status_message: None,
            error_message: None,
            output_text: String::new(),
            output_scroll: 0,
        }
    }
}

impl App {
    pub fn new() -> Result<Self> {
        let mut app = Self::default();

        // Load projects from project manager
        if let Ok(project_manager) = crate::project::ProjectManager::new() {
            if let Ok(projects) = project_manager.list_projects() {
                app.projects = projects.iter().map(|p| p.name.clone()).collect();
            }
        }

        Ok(app)
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn navigate_to(&mut self, screen: Screen) {
        self.current_screen = screen;
        self.input_mode = InputMode::Normal;
        self.clear_messages();
    }

    pub fn next_menu_item(&mut self) {
        self.selected_menu_item = (self.selected_menu_item + 1) % self.menu_items.len();
    }

    pub fn previous_menu_item(&mut self) {
        if self.selected_menu_item > 0 {
            self.selected_menu_item -= 1;
        } else {
            self.selected_menu_item = self.menu_items.len() - 1;
        }
    }

    pub fn select_current_menu_item(&mut self) {
        match self.selected_menu_item {
            0 => self.navigate_to(Screen::Prompt),
            1 => self.navigate_to(Screen::Configure),
            2 => self.navigate_to(Screen::Projects),
            3 => self.navigate_to(Screen::Models),
            4 => self.navigate_to(Screen::Settings),
            5 => self.navigate_to(Screen::Help),
            6 => self.quit(),
            _ => {}
        }
    }

    pub fn next_field(&mut self) {
        match self.current_screen {
            Screen::Prompt => {
                self.selected_field = (self.selected_field + 1) % self.prompt_fields.len();
            }
            Screen::Configure => {
                self.selected_config_field = (self.selected_config_field + 1) % self.config_fields.len();
            }
            Screen::Projects => {
                if !self.projects.is_empty() {
                    self.selected_project = (self.selected_project + 1) % self.projects.len();
                }
            }
            Screen::Settings => {
                self.selected_setting = (self.selected_setting + 1) % self.settings_items.len();
            }
            _ => {}
        }
    }

    pub fn previous_field(&mut self) {
        match self.current_screen {
            Screen::Prompt => {
                if self.selected_field > 0 {
                    self.selected_field -= 1;
                } else {
                    self.selected_field = self.prompt_fields.len() - 1;
                }
            }
            Screen::Configure => {
                if self.selected_config_field > 0 {
                    self.selected_config_field -= 1;
                } else {
                    self.selected_config_field = self.config_fields.len() - 1;
                }
            }
            Screen::Projects => {
                if !self.projects.is_empty() {
                    if self.selected_project > 0 {
                        self.selected_project -= 1;
                    } else {
                        self.selected_project = self.projects.len() - 1;
                    }
                }
            }
            Screen::Settings => {
                if self.selected_setting > 0 {
                    self.selected_setting -= 1;
                } else {
                    self.selected_setting = self.settings_items.len() - 1;
                }
            }
            _ => {}
        }
    }

    pub fn enter_input_mode(&mut self) {
        self.input_mode = InputMode::Editing;
    }

    pub fn exit_input_mode(&mut self) {
        self.input_mode = InputMode::Normal;
    }

    pub fn handle_char_input(&mut self, c: char) {
        match self.current_screen {
            Screen::Prompt => {
                if self.selected_field < self.prompt_fields.len() {
                    self.prompt_fields[self.selected_field].value.push(c);
                }
            }
            Screen::Configure => {
                if self.selected_config_field < self.config_fields.len() {
                    self.config_fields[self.selected_config_field].value.push(c);
                }
            }
            _ => {}
        }
    }

    pub fn handle_backspace(&mut self) {
        match self.current_screen {
            Screen::Prompt => {
                if self.selected_field < self.prompt_fields.len() {
                    self.prompt_fields[self.selected_field].value.pop();
                }
            }
            Screen::Configure => {
                if self.selected_config_field < self.config_fields.len() {
                    self.config_fields[self.selected_config_field].value.pop();
                }
            }
            _ => {}
        }
    }

    pub fn submit_prompt(&mut self) -> Result<()> {
        let goals = self.prompt_fields[0].value.clone();
        let format = if self.prompt_fields[1].value.is_empty() {
            "text".to_string()
        } else {
            self.prompt_fields[1].value.clone()
        };
        let warnings = self.prompt_fields[2].value.clone();

        if goals.is_empty() {
            self.error_message = Some("Goals cannot be empty".to_string());
            return Ok(());
        }

        self.status_message = Some("Submitting prompt...".to_string());

        // Call the prompt function (non-blocking would be better with tokio)
        match crate::prompt::structure_reasoning(&goals, &format, &warnings, false, None, false) {
            Ok(_) => {
                self.status_message = Some("Prompt submitted successfully!".to_string());
                // Clear form
                for field in &mut self.prompt_fields {
                    field.value.clear();
                }
            }
            Err(e) => {
                self.error_message = Some(format!("Error: {}", e));
            }
        }

        Ok(())
    }

    pub fn submit_configuration(&mut self) -> Result<()> {
        let provider = self.providers[self.selected_provider].clone();
        let api_key = self.config_fields[0].value.clone();
        let model = self.config_fields[1].value.clone();

        if api_key.is_empty() && provider != "Ollama" {
            self.error_message = Some("API Key is required for this provider".to_string());
            return Ok(());
        }

        // Create provider configuration
        let provider_config = crate::config::ProviderConfig {
            provider: provider.clone(),
            api_key,
            model: if model.is_empty() { None } else { Some(model) },
            additional_settings: None,
        };

        // Validate the configuration
        if let Err(e) = crate::config::validate_provider_config(&provider_config) {
            self.error_message = Some(format!("Invalid configuration: {}", e));
            return Ok(());
        }

        // Save configuration
        crate::config::add_provider(provider_config.clone());
        if let Err(e) = crate::config::save() {
            self.error_message = Some(format!("Failed to save configuration: {}", e));
            return Ok(());
        }

        self.status_message = Some(format!("Configuration saved for {}", provider));

        // Clear form
        for field in &mut self.config_fields {
            field.value.clear();
        }

        Ok(())
    }

    pub fn scroll_output_up(&mut self) {
        self.output_scroll = self.output_scroll.saturating_sub(1);
    }

    pub fn scroll_output_down(&mut self) {
        self.output_scroll = self.output_scroll.saturating_add(1);
    }

    pub fn clear_messages(&mut self) {
        self.status_message = None;
        self.error_message = None;
    }

    pub fn next_provider(&mut self) {
        self.selected_provider = (self.selected_provider + 1) % self.providers.len();
    }

    pub fn previous_provider(&mut self) {
        if self.selected_provider > 0 {
            self.selected_provider -= 1;
        } else {
            self.selected_provider = self.providers.len() - 1;
        }
    }
}
