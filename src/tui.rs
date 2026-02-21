use std::io::{self, Stdout};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
        KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};

use crate::{config, project::ProjectManager};

const TICK_RATE: Duration = Duration::from_millis(250);
const VIEWS: [&str; 4] = ["Overview", "Projects", "Models", "Keys"];

#[derive(Clone, Debug)]
struct ProjectRow {
    id: String,
    name: String,
    files: usize,
    goals: usize,
    contexts: usize,
    updated: String,
    is_active: bool,
}

#[derive(Clone, Debug, Default)]
struct DashboardData {
    active_provider: Option<String>,
    active_model: Option<String>,
    active_project: Option<String>,
    projects: Vec<ProjectRow>,
    config_warning: Option<String>,
    project_warning: Option<String>,
}

impl DashboardData {
    fn load() -> Self {
        let mut data = Self::default();

        match config::Config::load() {
            Ok(cfg) => {
                if let Some(provider) = cfg.get_active_provider() {
                    data.active_provider = Some(provider.provider);
                    data.active_model = provider.model;
                } else if cfg.active_provider.is_empty() {
                    data.config_warning =
                        Some("No active provider configured. Run `ola configure`.".to_string());
                } else {
                    data.active_provider = Some(cfg.active_provider.clone());
                    data.config_warning = Some(format!(
                        "Active provider '{}' exists, but credentials/model were not resolved.",
                        cfg.active_provider
                    ));
                }
            }
            Err(err) => {
                data.config_warning = Some(format!("Configuration unavailable: {}", err));
            }
        }

        match ProjectManager::new() {
            Ok(pm) => {
                let active_project_id = pm.get_active_project().ok().flatten();
                let mut active_name = None;

                match pm.list_projects() {
                    Ok(projects) => {
                        data.projects = projects
                            .into_iter()
                            .map(|project| {
                                let is_active =
                                    active_project_id.as_deref() == Some(project.id.as_str());
                                if is_active {
                                    active_name = Some(project.name.clone());
                                }

                                ProjectRow {
                                    id: project.id,
                                    name: project.name,
                                    files: project.files.len(),
                                    goals: project.goals.len(),
                                    contexts: project.contexts.len(),
                                    updated: project
                                        .updated_at
                                        .format("%Y-%m-%d %H:%M")
                                        .to_string(),
                                    is_active,
                                }
                            })
                            .collect();
                    }
                    Err(err) => {
                        data.project_warning = Some(format!("Could not load projects: {}", err));
                    }
                }

                data.active_project = active_name;
            }
            Err(err) => {
                data.project_warning = Some(format!("Project store unavailable: {}", err));
            }
        }

        data
    }

    fn provider_models(&self) -> &'static [&'static str] {
        match self.active_provider.as_deref() {
            Some("OpenAI") => &[
                "gpt-5",
                "gpt-4o",
                "gpt-4",
                "o3",
                "o3-pro",
                "o4-mini",
                "o4-mini-high",
            ],
            Some("Anthropic") => &[
                "claude-3-opus-20240229",
                "claude-3-sonnet-20240229",
                "claude-3-haiku-20240307",
                "claude-2.1",
            ],
            Some("Gemini") => &[
                "gemini-1.5-pro",
                "gemini-1.5-flash",
                "gemini-1.0-pro",
                "gemini-1.0-pro-vision",
            ],
            Some("Ollama") => &["Run `ola models --provider Ollama` to fetch live models"],
            _ => &["No provider configured"],
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum InputMode {
    Normal,
    Editing,
}

struct App {
    data: DashboardData,
    selected_view: usize,
    selected_project: usize,
    should_quit: bool,
    status: String,
    input_mode: InputMode,
    command_input: String,
    started_at: Instant,
}

impl App {
    fn new() -> Self {
        let data = DashboardData::load();
        let status = if let Some(message) = &data.config_warning {
            message.clone()
        } else {
            "Press `i` to enter command mode, `r` to refresh, `q` to quit.".to_string()
        };

        Self {
            data,
            selected_view: 0,
            selected_project: 0,
            should_quit: false,
            status,
            input_mode: InputMode::Normal,
            command_input: String::new(),
            started_at: Instant::now(),
        }
    }

    fn on_key(&mut self, key: KeyEvent) {
        match self.input_mode {
            InputMode::Normal => self.handle_normal_key(key),
            InputMode::Editing => self.handle_editing_key(key),
        }
    }

    fn handle_normal_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('c')) {
            self.should_quit = true;
            return;
        }

        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('r') => self.refresh(),
            KeyCode::Char('i') => self.input_mode = InputMode::Editing,
            KeyCode::Tab | KeyCode::Right => {
                self.selected_view = (self.selected_view + 1) % VIEWS.len()
            }
            KeyCode::BackTab | KeyCode::Left => {
                self.selected_view = if self.selected_view == 0 {
                    VIEWS.len() - 1
                } else {
                    self.selected_view - 1
                };
            }
            KeyCode::Up if self.selected_view == 1 => self.select_previous_project(),
            KeyCode::Down if self.selected_view == 1 => self.select_next_project(),
            KeyCode::Enter => self.activate_selection(),
            _ => {}
        }
    }

    fn handle_editing_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => self.input_mode = InputMode::Normal,
            KeyCode::Enter => {
                self.submit_command();
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Backspace => {
                self.command_input.pop();
            }
            KeyCode::Char(ch) => {
                self.command_input.push(ch);
            }
            _ => {}
        }
    }

    fn refresh(&mut self) {
        self.data = DashboardData::load();
        if self.selected_project >= self.data.projects.len() {
            self.selected_project = self.data.projects.len().saturating_sub(1);
        }
        self.status = "Dashboard refreshed.".to_string();
    }

    fn activate_selection(&mut self) {
        self.status = match self.selected_view {
            0 => {
                self.refresh();
                "Overview refreshed.".to_string()
            }
            1 => {
                if let Some(project) = self.selected_project() {
                    format!(
                        "Selected project `{}` (files: {}, goals: {}, contexts: {}).",
                        project.name, project.files, project.goals, project.contexts
                    )
                } else {
                    "No projects available. Use `ola project create --name <name>`.".to_string()
                }
            }
            2 => format!(
                "Model view focused. Active provider: {}",
                self.data
                    .active_provider
                    .as_deref()
                    .unwrap_or("not configured")
            ),
            _ => "Use `i` for command input, `r` to refresh, `q` to quit.".to_string(),
        };
    }

    fn submit_command(&mut self) {
        let command = self.command_input.trim().to_string();
        self.command_input.clear();

        if command.is_empty() {
            self.status = "No command entered.".to_string();
            return;
        }

        match command.as_str() {
            "refresh" => self.refresh(),
            "quit" | "exit" => self.should_quit = true,
            "overview" => {
                self.selected_view = 0;
                self.status = "Switched to Overview.".to_string();
            }
            "projects" => {
                self.selected_view = 1;
                self.status = "Switched to Projects.".to_string();
            }
            "models" => {
                self.selected_view = 2;
                self.status = "Switched to Models.".to_string();
            }
            "keys" => {
                self.selected_view = 3;
                self.status = "Switched to Keys.".to_string();
            }
            _ => {
                self.status = format!(
                    "Unknown command `{}`. Try: refresh, projects, models, overview, keys, quit.",
                    command
                );
            }
        }
    }

    fn select_previous_project(&mut self) {
        if self.data.projects.is_empty() {
            return;
        }

        self.selected_project = self.selected_project.saturating_sub(1);
    }

    fn select_next_project(&mut self) {
        if self.data.projects.is_empty() {
            return;
        }

        self.selected_project = (self.selected_project + 1).min(self.data.projects.len() - 1);
    }

    fn selected_project(&self) -> Option<&ProjectRow> {
        self.data.projects.get(self.selected_project)
    }
}

pub fn run() -> Result<()> {
    let mut terminal = setup_terminal()?;
    let run_result = run_app(&mut terminal);
    let restore_result = restore_terminal(&mut terminal);

    match (run_result, restore_result) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(run_err), Ok(())) => Err(run_err),
        (Ok(()), Err(restore_err)) => Err(restore_err),
        (Err(run_err), Err(restore_err)) => Err(anyhow::anyhow!("{}\n{}", run_err, restore_err)),
    }
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode().context("failed to enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .context("failed to switch to alternate screen")?;

    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend).context("failed to initialize terminal backend")
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode().context("failed to disable raw mode")?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .context("failed to restore terminal screen")?;
    terminal.show_cursor().context("failed to show cursor")?;
    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    let mut app = App::new();

    while !app.should_quit {
        terminal.draw(|frame| draw(frame, &app))?;

        if event::poll(TICK_RATE)? {
            if let Event::Key(key) = event::read()? {
                if matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
                    app.on_key(key);
                }
            }
        }
    }

    Ok(())
}

fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(4),
        ])
        .split(frame.area());

    draw_header(frame, chunks[0], app);
    draw_body(frame, chunks[1], app);
    draw_footer(frame, chunks[2], app);
}

fn draw_header(frame: &mut Frame, area: Rect, app: &App) {
    let provider = app
        .data
        .active_provider
        .as_deref()
        .unwrap_or("not configured");
    let uptime = app.started_at.elapsed().as_secs();

    let title = Line::from(vec![
        Span::styled(
            " Ola TUI ",
            Style::default().fg(Color::Black).bg(Color::Cyan),
        ),
        Span::raw(format!(
            "  Provider: {}  |  View: {}  |  Uptime: {}s",
            provider, VIEWS[app.selected_view], uptime
        )),
    ]);

    let header = Paragraph::new(title).block(Block::default().borders(Borders::ALL));
    frame.render_widget(header, area);
}

fn draw_body(frame: &mut Frame, area: Rect, app: &App) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(28), Constraint::Min(20)])
        .split(area);

    draw_view_selector(frame, columns[0], app);

    match app.selected_view {
        0 => draw_overview(frame, columns[1], app),
        1 => draw_projects(frame, columns[1], app),
        2 => draw_models(frame, columns[1], app),
        _ => draw_keys(frame, columns[1], app),
    }
}

fn draw_view_selector(frame: &mut Frame, area: Rect, app: &App) {
    let panels = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(7), Constraint::Length(7)])
        .split(area);

    let items: Vec<ListItem> = VIEWS
        .iter()
        .map(|view| ListItem::new(Line::from(format!("  {}", view))))
        .collect();

    let views = List::new(items)
        .block(Block::default().title("Views").borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">>");

    let mut state = ListState::default();
    state.select(Some(app.selected_view));
    frame.render_stateful_widget(views, panels[0], &mut state);

    let mode_label = match app.input_mode {
        InputMode::Normal => "NORMAL",
        InputMode::Editing => "EDIT",
    };
    let command_preview = if app.command_input.is_empty() {
        "<empty>"
    } else {
        app.command_input.as_str()
    };

    let command = Paragraph::new(vec![
        Line::from(format!("Mode: {}", mode_label)),
        Line::from("Press `i` to edit"),
        Line::from("Command:"),
        Line::from(command_preview),
    ])
    .block(Block::default().title("Input").borders(Borders::ALL))
    .wrap(Wrap { trim: true });

    frame.render_widget(command, panels[1]);
}

fn draw_overview(frame: &mut Frame, area: Rect, app: &App) {
    let provider = app
        .data
        .active_provider
        .as_deref()
        .unwrap_or("Not configured");
    let model = app.data.active_model.as_deref().unwrap_or("Not configured");
    let active_project = app
        .data
        .active_project
        .as_deref()
        .unwrap_or("No active project");

    let mut lines = vec![
        Line::from(format!("Active provider: {}", provider)),
        Line::from(format!("Active model: {}", model)),
        Line::from(format!("Projects discovered: {}", app.data.projects.len())),
        Line::from(format!("Active project: {}", active_project)),
        Line::from(""),
        Line::from("Use this view as an at-a-glance status dashboard."),
    ];

    if let Some(warning) = &app.data.config_warning {
        lines.push(Line::from(""));
        lines.push(Line::styled(
            format!("Config warning: {}", warning),
            Style::default().fg(Color::LightRed),
        ));
    }

    if let Some(warning) = &app.data.project_warning {
        lines.push(Line::styled(
            format!("Project warning: {}", warning),
            Style::default().fg(Color::LightRed),
        ));
    }

    let panel = Paragraph::new(lines)
        .block(Block::default().title("Overview").borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    frame.render_widget(panel, area);
}

fn draw_projects(frame: &mut Frame, area: Rect, app: &App) {
    if app.data.projects.is_empty() {
        let empty = Paragraph::new(
            "No projects found. Create one with `ola project create --name <name>`.",
        )
        .block(Block::default().title("Projects").borders(Borders::ALL))
        .wrap(Wrap { trim: true });
        frame.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = app
        .data
        .projects
        .iter()
        .map(|project| {
            let marker = if project.is_active { "*" } else { " " };
            ListItem::new(vec![
                Line::from(format!(
                    "{} {}  | files:{} goals:{} ctx:{}",
                    marker, project.name, project.files, project.goals, project.contexts
                )),
                Line::styled(
                    format!("  id:{}  updated:{}", project.id, project.updated),
                    Style::default().fg(Color::DarkGray),
                ),
            ])
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title("Projects (up/down + enter)")
                .borders(Borders::ALL),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">>");

    let mut state = ListState::default();
    state.select(Some(app.selected_project));
    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_models(frame: &mut Frame, area: Rect, app: &App) {
    let model_list = app.data.provider_models();
    let active_model = app.data.active_model.as_deref().unwrap_or("");

    let lines: Vec<Line> = model_list
        .iter()
        .map(|model| {
            if *model == active_model {
                Line::styled(
                    format!("* {}", model),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Line::from(format!("  {}", model))
            }
        })
        .collect();

    let panel = Paragraph::new(lines)
        .block(
            Block::default()
                .title("Models (provider-scoped)")
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(panel, area);
}

fn draw_keys(frame: &mut Frame, area: Rect, app: &App) {
    let mode = match app.input_mode {
        InputMode::Normal => "normal",
        InputMode::Editing => "editing",
    };
    let lines = vec![
        Line::from("q: Quit"),
        Line::from("r: Refresh dashboard data"),
        Line::from("Tab / Left / Right: Switch view"),
        Line::from("Up / Down: Move project cursor (Projects view)"),
        Line::from("Enter: Activate current selection"),
        Line::from("i: Enter command mode"),
        Line::from("Esc: Exit command mode"),
        Line::from(""),
        Line::from(format!(
            "Command mode accepts: refresh, overview, projects, models, keys, quit"
        )),
        Line::from(format!("Current mode: {}", mode)),
    ];

    let panel = Paragraph::new(lines)
        .block(
            Block::default()
                .title("Keyboard Shortcuts")
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(panel, area);
}

fn draw_footer(frame: &mut Frame, area: Rect, app: &App) {
    let status = Paragraph::new(vec![
        Line::from(app.status.clone()),
        Line::styled(
            "Hint: run `ola tui` directly from your terminal to launch this dashboard.",
            Style::default().fg(Color::DarkGray),
        ),
    ])
    .block(Block::default().title("Status").borders(Borders::ALL))
    .wrap(Wrap { trim: true });

    frame.render_widget(status, area);
}
