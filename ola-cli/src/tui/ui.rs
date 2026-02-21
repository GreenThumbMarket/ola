/// UI rendering for the TUI with ocean theme

use crate::tui::app::{App, AppMode, Screen};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap,
    },
    Frame,
};

// Ocean theme colors
const OCEAN_BLUE: Color = Color::Rgb(0, 119, 190);
const DEEP_BLUE: Color = Color::Rgb(0, 82, 165);
const LIGHT_BLUE: Color = Color::Rgb(135, 206, 235);
const WAVE_CYAN: Color = Color::Cyan;
const FOAM_WHITE: Color = Color::Rgb(240, 248, 255);

pub fn draw(f: &mut Frame, app: &App) {
    match app.screen {
        Screen::Main => draw_main_screen(f, app),
        Screen::ProviderSelection => draw_provider_selection(f, app),
        Screen::Settings => draw_settings_screen(f, app),
        Screen::Help => draw_help_screen(f, app),
    }

    // Draw status bar
    draw_status_bar(f, app);

    // Draw any error messages
    if let AppMode::Error(ref msg) = app.mode {
        draw_error_popup(f, msg);
    }
}

fn draw_main_screen(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(5),    // Chat messages
            Constraint::Length(4), // Input box
        ])
        .split(f.size());

    // Draw title with wave animation
    let title = draw_animated_title(app);
    let title_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(OCEAN_BLUE));
    let title_widget = Paragraph::new(title)
        .block(title_block)
        .alignment(Alignment::Center);
    f.render_widget(title_widget, chunks[0]);

    // Draw chat messages
    draw_chat_messages(f, app, chunks[1]);

    // Draw input box
    draw_input_box(f, app, chunks[2]);
}

fn draw_animated_title(app: &App) -> Line {
    let wave_frames = ["～", "≈", "∼", "～", "≈", "∼"];
    let frame = wave_frames[app.animation_frame % wave_frames.len()];

    let mut spans = vec![
        Span::styled(frame, Style::default().fg(WAVE_CYAN)),
        Span::raw(" "),
        Span::styled("🌊", Style::default()),
        Span::raw(" "),
        Span::styled("Ola AI Assistant", Style::default().fg(OCEAN_BLUE).add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::styled("🌊", Style::default()),
        Span::raw(" "),
        Span::styled(frame, Style::default().fg(WAVE_CYAN)),
    ];

    if app.is_thinking {
        let dots = ".".repeat(app.thinking_dots + 1);
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            format!("Thinking{}", dots),
            Style::default().fg(LIGHT_BLUE).add_modifier(Modifier::ITALIC),
        ));
    }

    Line::from(spans)
}

fn draw_chat_messages(f: &mut Frame, app: &App, area: Rect) {
    let messages_block = Block::default()
        .title(" Chat ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(DEEP_BLUE));

    let inner_area = messages_block.inner(area);
    f.render_widget(messages_block, area);

    if app.messages.is_empty() {
        let welcome = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("Welcome to ", Style::default().fg(LIGHT_BLUE)),
                Span::styled("Ola", Style::default().fg(OCEAN_BLUE).add_modifier(Modifier::BOLD)),
                Span::styled(" - Your AI Wave Assistant", Style::default().fg(LIGHT_BLUE)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Press ", Style::default().fg(Color::Gray)),
                Span::styled("i", Style::default().fg(WAVE_CYAN).add_modifier(Modifier::BOLD)),
                Span::styled(" to start typing, ", Style::default().fg(Color::Gray)),
                Span::styled("h", Style::default().fg(WAVE_CYAN).add_modifier(Modifier::BOLD)),
                Span::styled(" for help", Style::default().fg(Color::Gray)),
            ]),
        ];
        let welcome_widget = Paragraph::new(welcome)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });
        f.render_widget(welcome_widget, inner_area);
    } else {
        let mut lines: Vec<Line> = Vec::new();

        for msg in &app.messages {
            if msg.is_user {
                lines.push(Line::from(vec![
                    Span::styled("You: ", Style::default().fg(WAVE_CYAN).add_modifier(Modifier::BOLD)),
                    Span::raw(&msg.content),
                ]));
            } else {
                lines.push(Line::from(vec![
                    Span::styled("🌊 Ola: ", Style::default().fg(OCEAN_BLUE).add_modifier(Modifier::BOLD)),
                    Span::styled(&msg.content, Style::default().fg(FOAM_WHITE)),
                ]));
            }
            lines.push(Line::from(""));
        }

        let messages_widget = Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .scroll((app.scroll_offset as u16, 0));
        f.render_widget(messages_widget, inner_area);
    }
}

fn draw_input_box(f: &mut Frame, app: &App, area: Rect) {
    let input_style = match app.mode {
        AppMode::Input => Style::default().fg(WAVE_CYAN),
        _ => Style::default().fg(Color::Gray),
    };

    let input_block = Block::default()
        .title(match app.mode {
            AppMode::Input => " Input (ESC to exit) ",
            AppMode::Processing => " Processing... ",
            _ => " Press 'i' to input ",
        })
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(input_style);

    let input_widget = Paragraph::new(app.input.as_str())
        .block(input_block)
        .style(Style::default().fg(FOAM_WHITE))
        .wrap(Wrap { trim: false });

    f.render_widget(input_widget, area);

    // Show cursor in input mode
    if let AppMode::Input = app.mode {
        let cursor_x = area.x + 1 + app.input_cursor as u16;
        let cursor_y = area.y + 1;
        f.set_cursor(cursor_x, cursor_y);
    }
}

fn draw_provider_selection(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(f.size());

    // Title
    let title = Block::default()
        .title(" Provider Selection ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(OCEAN_BLUE));
    f.render_widget(title, chunks[0]);

    // Provider list
    let providers: Vec<ListItem> = app.providers
        .iter()
        .enumerate()
        .map(|(i, provider)| {
            let style = if i == app.selected_provider_index {
                Style::default().fg(WAVE_CYAN).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(FOAM_WHITE)
            };
            ListItem::new(provider.as_str()).style(style)
        })
        .collect();

    let providers_list = List::new(providers)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(Style::default().fg(DEEP_BLUE)))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_widget(providers_list, chunks[1]);

    // Instructions
    let instructions = Paragraph::new("↑/↓: Navigate  Enter: Select  ESC: Back")
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded))
        .alignment(Alignment::Center);
    f.render_widget(instructions, chunks[2]);
}

fn draw_settings_screen(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
        ])
        .split(f.size());

    let title = Block::default()
        .title(" Settings ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(OCEAN_BLUE));
    f.render_widget(title, chunks[0]);

    let settings_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Current Provider: ", Style::default().fg(LIGHT_BLUE)),
            Span::styled(
                app.config.as_ref()
                    .and_then(|c| c.get_active_provider())
                    .map(|p| p.provider.clone())
                    .unwrap_or_else(|| "Not configured".to_string()),
                Style::default().fg(WAVE_CYAN).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Model: ", Style::default().fg(LIGHT_BLUE)),
            Span::styled(&app.selected_model, Style::default().fg(WAVE_CYAN)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Wave Animation: ", Style::default().fg(LIGHT_BLUE)),
            Span::styled(
                if app.wave_animation { "Enabled" } else { "Disabled" },
                Style::default().fg(WAVE_CYAN),
            ),
        ]),
    ];

    let settings = Paragraph::new(settings_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(Style::default().fg(DEEP_BLUE)))
        .alignment(Alignment::Left);
    f.render_widget(settings, chunks[1]);
}

fn draw_help_screen(f: &mut Frame, _app: &App) {
    let help_text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "🌊 Ola TUI Help",
            Style::default().fg(OCEAN_BLUE).add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled("Navigation:", Style::default().fg(LIGHT_BLUE).add_modifier(Modifier::BOLD))]),
        Line::from(vec![
            Span::styled("  i     ", Style::default().fg(WAVE_CYAN)),
            Span::raw("- Enter input mode"),
        ]),
        Line::from(vec![
            Span::styled("  ESC   ", Style::default().fg(WAVE_CYAN)),
            Span::raw("- Exit input mode / Go back"),
        ]),
        Line::from(vec![
            Span::styled("  p     ", Style::default().fg(WAVE_CYAN)),
            Span::raw("- Provider selection"),
        ]),
        Line::from(vec![
            Span::styled("  s     ", Style::default().fg(WAVE_CYAN)),
            Span::raw("- Settings"),
        ]),
        Line::from(vec![
            Span::styled("  h/?   ", Style::default().fg(WAVE_CYAN)),
            Span::raw("- Help"),
        ]),
        Line::from(vec![
            Span::styled("  q     ", Style::default().fg(WAVE_CYAN)),
            Span::raw("- Quit"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled("Chat Controls:", Style::default().fg(LIGHT_BLUE).add_modifier(Modifier::BOLD))]),
        Line::from(vec![
            Span::styled("  ↑/↓   ", Style::default().fg(WAVE_CYAN)),
            Span::raw("- Scroll messages"),
        ]),
        Line::from(vec![
            Span::styled("  PgUp/PgDn ", Style::default().fg(WAVE_CYAN)),
            Span::raw("- Scroll faster"),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+C ", Style::default().fg(WAVE_CYAN)),
            Span::raw("- Clear chat"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled("Input Mode:", Style::default().fg(LIGHT_BLUE).add_modifier(Modifier::BOLD))]),
        Line::from(vec![
            Span::styled("  Enter  ", Style::default().fg(WAVE_CYAN)),
            Span::raw("- Send message"),
        ]),
        Line::from(vec![
            Span::styled("  ←/→   ", Style::default().fg(WAVE_CYAN)),
            Span::raw("- Move cursor"),
        ]),
        Line::from(vec![
            Span::styled("  Home/End ", Style::default().fg(WAVE_CYAN)),
            Span::raw("- Jump to start/end"),
        ]),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default()
            .title(" Help ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(Style::default().fg(DEEP_BLUE)))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false });

    f.render_widget(help, f.size());
}

fn draw_status_bar(f: &mut Frame, app: &App) {
    let status_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(20), Constraint::Length(30)])
        .split(Rect {
            x: f.size().x,
            y: f.size().height.saturating_sub(1),
            width: f.size().width,
            height: 1,
        });

    let mode_text = match &app.mode {
        AppMode::Normal => "NORMAL",
        AppMode::Input => "INPUT",
        AppMode::Processing => "PROCESSING",
        AppMode::Error(_) => "ERROR",
    };

    let status_text = if let Some(ref msg) = app.status_message {
        msg.clone()
    } else {
        format!(" Mode: {} | Screen: {:?}", mode_text, app.screen)
    };

    let status = Paragraph::new(status_text)
        .style(Style::default().bg(DEEP_BLUE).fg(FOAM_WHITE));
    f.render_widget(status, status_chunks[0]);
}

fn draw_error_popup(f: &mut Frame, message: &str) {
    let popup_area = centered_rect(60, 20, f.size());
    f.render_widget(Clear, popup_area);

    let popup = Paragraph::new(message)
        .block(Block::default()
            .title(" Error ")
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .style(Style::default().fg(Color::Red)))
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Center);

    f.render_widget(popup, popup_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}