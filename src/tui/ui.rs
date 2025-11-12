use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, BorderType, List, ListItem, Paragraph, Wrap},
    Frame,
};

use super::app::{App, InputMode, Screen};

/// Render the main UI
pub fn render(frame: &mut Frame, app: &App) {
    let size = frame.area();

    // Create main layout with header, body, and footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(0),     // Body
            Constraint::Length(3),  // Footer
        ])
        .split(size);

    // Render header
    render_header(frame, chunks[0], app);

    // Render appropriate screen
    match app.current_screen {
        Screen::Home => render_home(frame, chunks[1], app),
        Screen::Prompt => render_prompt(frame, chunks[1], app),
        Screen::Configure => render_configure(frame, chunks[1], app),
        Screen::Projects => render_projects(frame, chunks[1], app),
        Screen::Settings => render_settings(frame, chunks[1], app),
        Screen::Models => render_models(frame, chunks[1], app),
        Screen::Help => render_help(frame, chunks[1], app),
    }

    // Render footer
    render_footer(frame, chunks[2], app);
}

fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let title = match app.current_screen {
        Screen::Home => "Ola - AI CLI Tool",
        Screen::Prompt => "Ola - Prompt",
        Screen::Configure => "Ola - Configuration",
        Screen::Projects => "Ola - Projects",
        Screen::Settings => "Ola - Settings",
        Screen::Models => "Ola - Models",
        Screen::Help => "Ola - Help",
    };

    let header = Paragraph::new(title)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(Color::Cyan)),
        );

    frame.render_widget(header, area);
}

fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let key_hints = match app.input_mode {
        InputMode::Normal => match app.current_screen {
            Screen::Home => vec![
                ("↑↓/jk", "Navigate"),
                ("Enter", "Select"),
                ("1-6", "Quick Nav"),
                ("q/Esc", "Quit"),
                ("?", "Help"),
            ],
            Screen::Prompt | Screen::Configure => vec![
                ("↑↓/jk", "Navigate"),
                ("Tab", "Next Field"),
                ("Enter/i/e", "Edit"),
                ("Ctrl+S", "Submit"),
                ("Esc", "Back"),
            ],
            Screen::Projects | Screen::Settings => vec![
                ("↑↓/jk", "Navigate"),
                ("Enter", "Select"),
                ("Esc", "Back"),
                ("?", "Help"),
            ],
            _ => vec![
                ("↑↓/jk", "Navigate"),
                ("Esc", "Back"),
                ("q", "Quit"),
            ],
        },
        InputMode::Editing => vec![
            ("Type", "Input"),
            ("Enter/Esc", "Done"),
            ("Backspace", "Delete"),
        ],
    };

    let footer_text: Vec<Span> = key_hints
        .iter()
        .flat_map(|(key, desc)| {
            vec![
                Span::styled(*key, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(": "),
                Span::raw(*desc),
                Span::raw(" | "),
            ]
        })
        .collect();

    let footer = Paragraph::new(Line::from(footer_text))
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(Color::DarkGray)),
        );

    frame.render_widget(footer, area);
}

fn render_home(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(5)])
        .split(area);

    // Menu
    let items: Vec<ListItem> = app
        .menu_items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == app.selected_menu_item {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let icon = match *item {
                "Prompt" => "📝",
                "Configure" => "⚙️",
                "Projects" => "📁",
                "Models" => "🤖",
                "Settings" => "🔧",
                "Help" => "❓",
                "Quit" => "🚪",
                _ => "•",
            };

            ListItem::new(format!("  {} {}", icon, item)).style(style)
        })
        .collect();

    let menu = List::new(items)
        .block(
            Block::default()
                .title(" Main Menu ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(Color::Cyan)),
        );

    frame.render_widget(menu, chunks[0]);

    // Welcome message
    let welcome = Paragraph::new(
        "Welcome to Ola! Use arrow keys or j/k to navigate, Enter to select.\nPress 1-6 for quick navigation or ? for help.",
    )
    .style(Style::default().fg(Color::Gray))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(Style::default().fg(Color::DarkGray)),
    )
    .wrap(Wrap { trim: true });

    frame.render_widget(welcome, chunks[1]);
}

fn render_prompt(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    // Form fields
    let form_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Length(5); app.prompt_fields.len()])
        .split(chunks[0]);

    for (i, field) in app.prompt_fields.iter().enumerate() {
        let is_selected = i == app.selected_field;
        let is_editing = is_selected && app.input_mode == InputMode::Editing;

        let style = if is_editing {
            Style::default().fg(Color::Yellow)
        } else if is_selected {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::White)
        };

        let border_style = if is_editing {
            Style::default().fg(Color::Yellow)
        } else if is_selected {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let display_text = if field.value.is_empty() {
            format!("({})", field.placeholder)
        } else {
            field.value.clone()
        };

        let cursor = if is_editing { "█" } else { "" };
        let text = format!("{}{}", display_text, cursor);

        let paragraph = Paragraph::new(text)
            .style(style)
            .block(
                Block::default()
                    .title(format!(" {} ", field.label))
                    .borders(Borders::ALL)
                    .border_type(if is_editing {
                        BorderType::Double
                    } else {
                        BorderType::Rounded
                    })
                    .style(border_style),
            )
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, form_chunks[i]);
    }

    // Status or error message
    render_status_area(frame, chunks[1], app);
}

fn render_configure(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),  // Provider selection
            Constraint::Min(0),     // Form fields
            Constraint::Length(3),  // Status
        ])
        .split(area);

    // Provider selection
    let provider_items: Vec<Span> = app
        .providers
        .iter()
        .enumerate()
        .flat_map(|(i, provider)| {
            let style = if i == app.selected_provider {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            vec![
                Span::styled(format!(" {} ", provider), style),
                Span::raw("  "),
            ]
        })
        .collect();

    let provider_selection = Paragraph::new(Line::from(provider_items))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title(" Select Provider (←/→ or h/l) ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(Color::Cyan)),
        );

    frame.render_widget(provider_selection, chunks[0]);

    // Form fields
    let form_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Length(5); app.config_fields.len()])
        .split(chunks[1]);

    for (i, field) in app.config_fields.iter().enumerate() {
        let is_selected = i == app.selected_config_field;
        let is_editing = is_selected && app.input_mode == InputMode::Editing;

        let style = if is_editing {
            Style::default().fg(Color::Yellow)
        } else if is_selected {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::White)
        };

        let border_style = if is_editing {
            Style::default().fg(Color::Yellow)
        } else if is_selected {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let display_text = if field.value.is_empty() {
            format!("({})", field.placeholder)
        } else if field.label.contains("Key") || field.label.contains("key") {
            "*".repeat(field.value.len())
        } else {
            field.value.clone()
        };

        let cursor = if is_editing { "█" } else { "" };
        let text = format!("{}{}", display_text, cursor);

        let paragraph = Paragraph::new(text)
            .style(style)
            .block(
                Block::default()
                    .title(format!(" {} ", field.label))
                    .borders(Borders::ALL)
                    .border_type(if is_editing {
                        BorderType::Double
                    } else {
                        BorderType::Rounded
                    })
                    .style(border_style),
            )
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, form_chunks[i]);
    }

    // Status area
    render_status_area(frame, chunks[2], app);
}

fn render_projects(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(5)])
        .split(area);

    // Projects list
    if app.projects.is_empty() {
        let empty_msg = Paragraph::new("No projects found.\n\nUse 'ola project create' from the CLI to create a new project.")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .title(" Projects ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .style(Style::default().fg(Color::Cyan)),
            )
            .wrap(Wrap { trim: true });

        frame.render_widget(empty_msg, chunks[0]);
    } else {
        let items: Vec<ListItem> = app
            .projects
            .iter()
            .enumerate()
            .map(|(i, project)| {
                let style = if i == app.selected_project {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                ListItem::new(format!("  📁 {}", project)).style(style)
            })
            .collect();

        let projects_list = List::new(items).block(
            Block::default()
                .title(" Projects ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(Color::Cyan)),
        );

        frame.render_widget(projects_list, chunks[0]);
    }

    // Help text
    let help_text = Paragraph::new("Use CLI commands for project management:\n• Create: ola project create --name <name>\n• Delete: ola project delete --project <name>\n• View: ola project show")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(Color::DarkGray)),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(help_text, chunks[1]);
}

fn render_settings(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(5)])
        .split(area);

    // Settings list
    let items: Vec<ListItem> = app
        .settings_items
        .iter()
        .enumerate()
        .map(|(i, setting)| {
            let style = if i == app.selected_setting {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(format!("  🔧 {}", setting)).style(style)
        })
        .collect();

    let settings_list = List::new(items).block(
        Block::default()
            .title(" Settings ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(settings_list, chunks[0]);

    // Help text
    let help_text = Paragraph::new("Use CLI commands for settings management:\n• View: ola settings --view\n• Set default model: ola settings --default-model <model>\n• Enable logging: ola settings --logging true")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(Color::DarkGray)),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(help_text, chunks[1]);
}

fn render_models(frame: &mut Frame, area: Rect, app: &App) {
    let models_text = r#"Available Models:

OpenAI:
  • gpt-5
  • gpt-4o
  • gpt-4
  • o3, o3-pro
  • o4, o4-mini, o4-mini-high

Anthropic:
  • claude-3-opus-20240229
  • claude-3-sonnet-20240229
  • claude-3-haiku-20240307
  • claude-2.1, claude-2.0

Gemini:
  • gemini-1.5-pro
  • gemini-1.5-flash
  • gemini-1.0-pro
  • gemini-1.0-pro-vision

Ollama:
  • Use 'ola models --provider Ollama' to see locally available models

Note: Use 'ola configure' to set up your preferred provider and model."#;

    let models = Paragraph::new(models_text)
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .title(" Available Models ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: true })
        .scroll((app.output_scroll, 0));

    frame.render_widget(models, area);
}

fn render_help(frame: &mut Frame, area: Rect, app: &App) {
    let help_text = r#"Ola - AI CLI Tool TUI Help

NAVIGATION:
  • Arrow Keys / j,k,h,l: Navigate menus and lists
  • Enter / Space: Select / Activate
  • Tab / Shift+Tab: Cycle through form fields
  • Esc: Go back / Exit edit mode
  • q: Quit (from home screen)
  • Ctrl+C: Force quit
  • ?: Show this help screen
  • 1-6: Quick navigation (from home)

SCREENS:
  1. Home: Main menu for navigation
  2. Prompt: Submit prompts with goals, format, warnings
  3. Configure: Set up provider, API key, and model
  4. Projects: View and manage projects
  5. Models: View available models for each provider
  6. Settings: View and modify application settings

PROMPT SCREEN:
  • Use Tab or j/k to navigate fields
  • Press Enter/i/e to edit a field
  • Press Esc when done editing
  • Press Ctrl+S to submit the prompt

CONFIGURE SCREEN:
  • Use h/l or ←/→ to select provider
  • Use Tab or j/k to navigate fields
  • Press Enter/i/e to edit a field
  • Press Esc when done editing
  • Press Ctrl+S to save configuration

TIPS:
  • Most CLI commands are still available alongside the TUI
  • Use 'ola --help' for CLI documentation
  • Configuration is shared between TUI and CLI modes
  • Projects created via CLI are visible in TUI

Press Esc to return to the previous screen."#;

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .title(" Help ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: true })
        .scroll((app.output_scroll, 0));

    frame.render_widget(help, area);
}

fn render_status_area(frame: &mut Frame, area: Rect, app: &App) {
    let (text, style) = if let Some(ref error) = app.error_message {
        (format!("❌ Error: {}", error), Style::default().fg(Color::Red))
    } else if let Some(ref status) = app.status_message {
        (format!("✅ {}", status), Style::default().fg(Color::Green))
    } else {
        ("Ready. Press Ctrl+S to submit.".to_string(), Style::default().fg(Color::Gray))
    };

    let status = Paragraph::new(text)
        .style(style)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(Color::DarkGray)),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(status, area);
}
