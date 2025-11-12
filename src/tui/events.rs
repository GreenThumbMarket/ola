use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

use super::app::{App, InputMode, Screen};

pub struct EventHandler;

impl EventHandler {
    pub fn new() -> Self {
        Self
    }

    /// Handle terminal events and update app state
    pub fn handle_events(&self, app: &mut App) -> Result<()> {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                self.handle_key_event(key, app)?;
            }
        }
        Ok(())
    }

    fn handle_key_event(&self, key: KeyEvent, app: &mut App) -> Result<()> {
        match app.input_mode {
            InputMode::Normal => self.handle_normal_mode(key, app),
            InputMode::Editing => self.handle_editing_mode(key, app),
        }
    }

    fn handle_normal_mode(&self, key: KeyEvent, app: &mut App) -> Result<()> {
        match key.code {
            // Global keybindings
            KeyCode::Char('q') | KeyCode::Esc => {
                if app.current_screen == Screen::Home {
                    app.quit();
                } else {
                    app.navigate_to(Screen::Home);
                }
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.quit();
            }
            KeyCode::Char('?') => {
                app.navigate_to(Screen::Help);
            }

            // Navigation based on current screen
            _ => match app.current_screen {
                Screen::Home => self.handle_home_keys(key, app)?,
                Screen::Prompt => self.handle_prompt_keys(key, app)?,
                Screen::Configure => self.handle_configure_keys(key, app)?,
                Screen::Projects => self.handle_projects_keys(key, app)?,
                Screen::Settings => self.handle_settings_keys(key, app)?,
                Screen::Models => self.handle_models_keys(key, app)?,
                Screen::Help => self.handle_help_keys(key, app)?,
            },
        }
        Ok(())
    }

    fn handle_editing_mode(&self, key: KeyEvent, app: &mut App) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                app.exit_input_mode();
            }
            KeyCode::Enter => {
                app.exit_input_mode();
            }
            KeyCode::Char(c) => {
                app.handle_char_input(c);
            }
            KeyCode::Backspace => {
                app.handle_backspace();
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_home_keys(&self, key: KeyEvent, app: &mut App) -> Result<()> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                app.previous_menu_item();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.next_menu_item();
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                app.select_current_menu_item();
            }
            KeyCode::Char('1') => app.navigate_to(Screen::Prompt),
            KeyCode::Char('2') => app.navigate_to(Screen::Configure),
            KeyCode::Char('3') => app.navigate_to(Screen::Projects),
            KeyCode::Char('4') => app.navigate_to(Screen::Models),
            KeyCode::Char('5') => app.navigate_to(Screen::Settings),
            KeyCode::Char('6') => app.navigate_to(Screen::Help),
            _ => {}
        }
        Ok(())
    }

    fn handle_prompt_keys(&self, key: KeyEvent, app: &mut App) -> Result<()> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                app.previous_field();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.next_field();
            }
            KeyCode::Tab => {
                app.next_field();
            }
            KeyCode::BackTab => {
                app.previous_field();
            }
            KeyCode::Enter | KeyCode::Char('i') | KeyCode::Char('e') => {
                app.enter_input_mode();
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.submit_prompt()?;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_configure_keys(&self, key: KeyEvent, app: &mut App) -> Result<()> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                app.previous_field();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.next_field();
            }
            KeyCode::Left | KeyCode::Char('h') => {
                app.previous_provider();
            }
            KeyCode::Right | KeyCode::Char('l') => {
                app.next_provider();
            }
            KeyCode::Tab => {
                app.next_field();
            }
            KeyCode::BackTab => {
                app.previous_field();
            }
            KeyCode::Enter | KeyCode::Char('i') | KeyCode::Char('e') => {
                app.enter_input_mode();
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.submit_configuration()?;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_projects_keys(&self, key: KeyEvent, app: &mut App) -> Result<()> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                app.previous_field();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.next_field();
            }
            KeyCode::Char('n') => {
                app.status_message = Some("Create new project: Use 'ola project create' from CLI".to_string());
            }
            KeyCode::Char('d') => {
                if !app.projects.is_empty() {
                    app.status_message = Some(format!("Delete project: Use 'ola project delete' from CLI"));
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_settings_keys(&self, key: KeyEvent, app: &mut App) -> Result<()> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                app.previous_field();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.next_field();
            }
            KeyCode::Enter => {
                app.status_message = Some("Editing settings: Use 'ola settings' from CLI".to_string());
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_models_keys(&self, key: KeyEvent, app: &mut App) -> Result<()> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                app.scroll_output_up();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.scroll_output_down();
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_help_keys(&self, key: KeyEvent, app: &mut App) -> Result<()> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                app.scroll_output_up();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.scroll_output_down();
            }
            _ => {}
        }
        Ok(())
    }
}
