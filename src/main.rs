// Cargo.toml dependencies:
// [dependencies]
// tokio = { version = "1.40", features = ["full"] }
// ratatui = "0.28"
// crossterm = "0.28"
// bollard = "0.17"
// chrono = "0.4"
// anyhow = "1.0"
// futures = "0.3"

use anyhow::Result;
use bollard::container::{ListContainersOptions, LogsOptions, StatsOptions};
use bollard::Docker;
use chrono::Utc;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::StreamExt;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table, TableState},
    Frame, Terminal,
};
use std::io;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
struct ContainerInfo {
    id: String,
    short_id: String,
    name: String,
    status: String,
    image: String,
    ports: String,
    created: i64,
    state: String,
}

#[derive(Debug, Clone)]
struct ContainerStats {
    cpu_percent: f64,
    memory_usage: u64,
    memory_limit: u64,
}

enum AppView {
    ContainerList,
    ContainerLogs(String, String),
}

struct App {
    docker: Docker,
    containers: Arc<RwLock<Vec<ContainerInfo>>>,
    container_stats: Arc<RwLock<std::collections::HashMap<String, ContainerStats>>>,
    table_state: TableState,
    show_all: bool,
    last_update: Instant,
    view: AppView,
    logs: Vec<String>,
    total_containers: usize,
    running_count: usize,
    stopped_count: usize,
    paused_count: usize,
}

impl App {
    async fn new() -> Result<Self> {
        let docker = Docker::connect_with_local_defaults()?;
        let containers = Arc::new(RwLock::new(Vec::new()));
        let container_stats = Arc::new(RwLock::new(std::collections::HashMap::new()));
        
        let mut app = Self {
            docker,
            containers: containers.clone(),
            container_stats: container_stats.clone(),
            table_state: TableState::default(),
            show_all: true,
            last_update: Instant::now(),
            view: AppView::ContainerList,
            logs: Vec::new(),
            total_containers: 0,
            running_count: 0,
            stopped_count: 0,
            paused_count: 0,
        };
        
        app.refresh_containers().await?;
        if app.total_containers > 0 {
            app.table_state.select(Some(0));
        }
        
        // Spawn background task for stats updates
        let docker_clone = app.docker.clone();
        let containers_clone = containers.clone();
        let stats_clone = container_stats.clone();
        
        tokio::spawn(async move {
            loop {
                let containers = containers_clone.read().await;
                let running_containers: Vec<_> = containers
                    .iter()
                    .filter(|c| c.state == "running")
                    .map(|c| c.id.clone())
                    .collect();
                drop(containers);
                
                // Fetch stats concurrently for all running containers
                let stats_futures: Vec<_> = running_containers
                    .iter()
                    .map(|id| {
                        let docker = docker_clone.clone();
                        let id = id.clone();
                        async move {
                            let mut stats_stream = docker.stats(
                                &id,
                                Some(StatsOptions {
                                    stream: false,
                                    ..Default::default()
                                }),
                            );
                            
                            if let Some(Ok(stats)) = stats_stream.next().await {
                                let cpu_delta = stats.cpu_stats.cpu_usage.total_usage
                                    .saturating_sub(stats.precpu_stats.cpu_usage.total_usage);
                                let system_delta = stats
                                    .cpu_stats
                                    .system_cpu_usage
                                    .unwrap_or(0)
                                    .saturating_sub(stats.precpu_stats.system_cpu_usage.unwrap_or(0));

                                let cpu_percent = if system_delta > 0 && cpu_delta > 0 {
                                    let num_cpus = stats
                                        .cpu_stats
                                        .online_cpus
                                        .unwrap_or_else(|| {
                                            stats
                                                .cpu_stats
                                                .cpu_usage
                                                .percpu_usage
                                                .as_ref()
                                                .map(|p| p.len() as u64)
                                                .unwrap_or(1)
                                        });
                                    (cpu_delta as f64 / system_delta as f64) * num_cpus as f64 * 100.0
                                } else {
                                    0.0
                                };

                                let memory_usage = stats.memory_stats.usage.unwrap_or(0);
                                let memory_limit = stats.memory_stats.limit.unwrap_or(0);

                                Some((
                                    id,
                                    ContainerStats {
                                        cpu_percent,
                                        memory_usage,
                                        memory_limit,
                                    },
                                ))
                            } else {
                                None
                            }
                        }
                    })
                    .collect();
                
                let results = futures::future::join_all(stats_futures).await;
                
                let mut stats_map = stats_clone.write().await;
                for result in results.into_iter().flatten() {
                    stats_map.insert(result.0, result.1);
                }
                drop(stats_map);
                
                tokio::time::sleep(Duration::from_secs(3)).await;
            }
        });
        
        Ok(app)
    }

    async fn refresh_containers(&mut self) -> Result<()> {
        let mut filters = std::collections::HashMap::new();
        if !self.show_all {
            filters.insert("status".to_string(), vec!["running".to_string()]);
        }

        let options = ListContainersOptions {
            all: self.show_all,
            filters,
            ..Default::default()
        };

        let containers_result = self.docker.list_containers(Some(options)).await?;

        self.total_containers = containers_result.len();
        self.running_count = 0;
        self.stopped_count = 0;
        self.paused_count = 0;

        let new_containers: Vec<ContainerInfo> = containers_result
            .into_iter()
            .map(|c| {
                let state = c.state.as_deref().unwrap_or("unknown");
                match state {
                    "running" => self.running_count += 1,
                    "exited" => self.stopped_count += 1,
                    "paused" => self.paused_count += 1,
                    _ => {}
                }

                let ports = c
                    .ports
                    .as_ref()
                    .map(|p| {
                        p.iter()
                            .take(2)
                            .filter_map(|port| {
                                if let Some(public) = port.public_port {
                                    Some(format!("{}→{}", public, port.private_port))
                                } else {
                                    Some(port.private_port.to_string())
                                }
                            })
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or_default();

                ContainerInfo {
                    id: c.id.clone().unwrap_or_default(),
                    short_id: c
                        .id
                        .as_ref()
                        .map(|id| id.chars().take(12).collect())
                        .unwrap_or_default(),
                    name: c
                        .names
                        .as_ref()
                        .and_then(|n| n.first())
                        .map(|n| n.trim_start_matches('/').to_string())
                        .unwrap_or_default(),
                    status: c.status.unwrap_or_default(),
                    image: c.image.unwrap_or_default(),
                    ports,
                    created: c.created.unwrap_or(0),
                    state: state.to_string(),
                }
            })
            .collect();

        let mut containers = self.containers.write().await;
        *containers = new_containers;
        drop(containers);

        self.last_update = Instant::now();
        Ok(())
    }

    fn next(&mut self) {
        if self.total_containers == 0 {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= self.total_containers - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    fn previous(&mut self) {
        if self.total_containers == 0 {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.total_containers - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    async fn selected_container(&self) -> Option<ContainerInfo> {
        let containers = self.containers.read().await;
        self.table_state
            .selected()
            .and_then(|i| containers.get(i).cloned())
    }

    async fn restart_container(&mut self) -> Result<()> {
        if let Some(container) = self.selected_container().await {
            self.docker.restart_container(&container.id, None).await?;
        }
        Ok(())
    }

    async fn stop_container(&mut self) -> Result<()> {
        if let Some(container) = self.selected_container().await {
            self.docker.stop_container(&container.id, None).await?;
        }
        Ok(())
    }

    async fn start_container(&mut self) -> Result<()> {
        if let Some(container) = self.selected_container().await {
            self.docker
                .start_container::<String>(&container.id, None)
                .await?;
        }
        Ok(())
    }

    async fn remove_container(&mut self) -> Result<()> {
        if let Some(container) = self.selected_container().await {
            use bollard::container::RemoveContainerOptions;
            self.docker
                .remove_container(
                    &container.id,
                    Some(RemoveContainerOptions {
                        force: true,
                        ..Default::default()
                    }),
                )
                .await?;
            self.refresh_containers().await?;
        }
        Ok(())
    }

    fn toggle_filter(&mut self) {
        self.show_all = !self.show_all;
    }

    async fn show_logs(&mut self) -> Result<()> {
        if let Some(container) = self.selected_container().await {
            let container_id = container.id.clone();
            let container_name = container.name.clone();

            self.logs.clear();

            let options = LogsOptions::<String> {
                stdout: true,
                stderr: true,
                tail: "100".to_string(),
                timestamps: true,
                ..Default::default()
            };

            let mut log_stream = self.docker.logs(&container_id, Some(options));

            while let Some(log) = log_stream.next().await {
                if let Ok(log) = log {
                    self.logs.push(log.to_string());
                }
            }

            self.view = AppView::ContainerLogs(container_id, container_name);
        }
        Ok(())
    }

    fn exit_logs(&mut self) {
        self.view = AppView::ContainerList;
        self.logs.clear();
    }
}

fn format_bytes(bytes: u64) -> String {
    const GB: u64 = 1024 * 1024 * 1024;
    const MB: u64 = 1024 * 1024;

    if bytes >= GB {
        format!("{:.1}G", bytes / GB)
    } else if bytes >= MB {
        format!("{}M", bytes / MB)
    } else {
        format!("{}K", bytes / 1024)
    }
}

fn format_uptime(created: i64) -> String {
    let now = Utc::now().timestamp();
    let delta = now - created;

    let days = delta / 86400;
    let hours = (delta % 86400) / 3600;
    let minutes = (delta % 3600) / 60;

    if days > 0 {
        format!("{}d{}h", days, hours)
    } else if hours > 0 {
        format!("{}h{}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}

async fn draw_container_list(f: &mut Frame<'_>, app: &App) {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    // Stats bar
    let stats_text = vec![Line::from(vec![
        Span::styled("Containers: ", Style::default().fg(Color::Cyan).bold()),
        Span::styled(
            format!("{} ", app.total_containers),
            Style::default().fg(Color::White).bold(),
        ),
        Span::styled("Running: ", Style::default().fg(Color::Green).bold()),
        Span::styled(
            format!("{} ", app.running_count),
            Style::default().fg(Color::White).bold(),
        ),
        Span::styled("Stopped: ", Style::default().fg(Color::Red).bold()),
        Span::styled(
            format!("{} ", app.stopped_count),
            Style::default().fg(Color::White).bold(),
        ),
        Span::styled(" │ ", Style::default().dim()),
        Span::styled(
            format!(
                "Filter: {} ",
                if app.show_all { "all" } else { "running" }
            ),
            Style::default().dim().italic(),
        ),
    ])];

    let stats_block = Paragraph::new(stats_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Dockyard ")
                .title_alignment(Alignment::Center),
        )
        .alignment(Alignment::Left);
    f.render_widget(stats_block, chunks[0]);

    // Container table
    let containers = app.containers.read().await;
    let stats_map = app.container_stats.read().await;

    let header_cells = ["ID", "NAME", "STATUS", "IMAGE", "PORTS", "CPU%", "MEM", "UP"]
        .iter()
        .map(|h| {
            Cell::from(*h).style(Style::default().fg(Color::Black).bg(Color::Cyan).bold())
        });
    let header = Row::new(header_cells).height(1);

    let rows = containers.iter().map(|c| {
        let (status_symbol, status_color) = match c.state.as_str() {
            "running" => ("●", Color::Green),
            "exited" => ("■", Color::Red),
            "paused" => ("‖", Color::Yellow),
            _ => ("○", Color::Gray),
        };

        let (cpu_str, mem_str) = if c.state == "running" {
            if let Some(stats) = stats_map.get(&c.id) {
                (
                    format!("{:.0}", stats.cpu_percent),
                    format_bytes(stats.memory_usage),
                )
            } else {
                ("-".to_string(), "-".to_string())
            }
        } else {
            ("-".to_string(), "-".to_string())
        };

        let uptime = if c.state == "running" {
            format_uptime(c.created)
        } else {
            "-".to_string()
        };

        let cells = vec![
            Cell::from(c.short_id.clone()),
            Cell::from(c.name.clone()).style(Style::default().fg(Color::Cyan)),
            Cell::from(format!("{} {}", status_symbol, c.state))
                .style(Style::default().fg(status_color).bold()),
            Cell::from(c.image.chars().take(25).collect::<String>()),
            Cell::from(c.ports.chars().take(12).collect::<String>()),
            Cell::from(cpu_str),
            Cell::from(mem_str),
            Cell::from(uptime),
        ];
        Row::new(cells).height(1)
    });

    let table = Table::new(
        rows,
        [
            Constraint::Length(12),
            Constraint::Length(18),
            Constraint::Length(11),
            Constraint::Length(26),
            Constraint::Length(13),
            Constraint::Length(5),
            Constraint::Length(7),
            Constraint::Length(7),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta))
            .title(" Containers ")
            .title_alignment(Alignment::Center),
    )
    .highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol("▶ ");

    f.render_stateful_widget(table, chunks[1], &mut app.table_state.clone());

    // Footer
    let footer_text = vec![Line::from(vec![
        Span::styled("↑↓", Style::default().fg(Color::Cyan).bold()),
        Span::raw(" Nav  "),
        Span::styled("l", Style::default().fg(Color::Green).bold()),
        Span::raw(" Logs  "),
        Span::styled("r", Style::default().fg(Color::Yellow).bold()),
        Span::raw(" Restart  "),
        Span::styled("s", Style::default().fg(Color::Red).bold()),
        Span::raw(" Stop  "),
        Span::styled("t", Style::default().fg(Color::Green).bold()),
        Span::raw(" Start  "),
        Span::styled("d", Style::default().fg(Color::Magenta).bold()),
        Span::raw(" Remove  "),
        Span::styled("f", Style::default().fg(Color::Cyan).bold()),
        Span::raw(" Filter  "),
        Span::styled("q", Style::default().fg(Color::Red).bold()),
        Span::raw(" Quit"),
    ])];

    let footer = Paragraph::new(footer_text)
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[2]);
}

fn draw_logs_view(f: &mut Frame<'_>, app: &App, container_name: &str) {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    // Header
    let header = Paragraph::new(format!("Container: {}", container_name))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Logs ")
                .title_alignment(Alignment::Center),
        )
        .style(Style::default().fg(Color::Cyan).bold())
        .alignment(Alignment::Center);
    f.render_widget(header, chunks[0]);

    // Logs - show last 100 lines to avoid memory issues
    let start_idx = app.logs.len().saturating_sub(100);
    let logs_items: Vec<ListItem> = app.logs[start_idx..]
        .iter()
        .map(|log| ListItem::new(log.as_str()))
        .collect();

    let logs_list = List::new(logs_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta))
                .title(" Output ")
                .title_alignment(Alignment::Center),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(logs_list, chunks[1]);

    // Footer
    let footer = Paragraph::new("Press q or ESC to return")
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[2]);
}

#[tokio::main]
async fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new().await?;
    let mut last_container_update = Instant::now();

    loop {
        // Refresh container list every 15 seconds (reduced frequency)
        if last_container_update.elapsed() > Duration::from_secs(15) {
            let _ = app.refresh_containers().await;
            last_container_update = Instant::now();
        }

        // Draw UI
        match &app.view {
            AppView::ContainerList => {
                terminal.draw(|f| {
                    let app_ref = &app;
                    tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(draw_container_list(f, app_ref))
                    });
                })?;
            }
            AppView::ContainerLogs(_, name) => {
                let name = name.clone();
                terminal.draw(|f| draw_logs_view(f, &app, &name))?;
            }
        }

        // Poll for events with longer timeout for less CPU usage
        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match &app.view {
                        AppView::ContainerList => match key.code {
                            KeyCode::Char('q') => break,
                            KeyCode::Down | KeyCode::Char('j') => app.next(),
                            KeyCode::Up | KeyCode::Char('k') => app.previous(),
                            KeyCode::Char('r') => {
                                let _ = app.restart_container().await;
                                let _ = app.refresh_containers().await;
                            }
                            KeyCode::Char('s') => {
                                let _ = app.stop_container().await;
                                let _ = app.refresh_containers().await;
                            }
                            KeyCode::Char('t') => {
                                let _ = app.start_container().await;
                                let _ = app.refresh_containers().await;
                            }
                            KeyCode::Char('d') => {
                                let _ = app.remove_container().await;
                            }
                            KeyCode::Char('f') => {
                                app.toggle_filter();
                                let _ = app.refresh_containers().await;
                            }
                            KeyCode::Char('l') => {
                                let _ = app.show_logs().await;
                            }
                            _ => {}
                        },
                        AppView::ContainerLogs(_, _) => match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => app.exit_logs(),
                            _ => {}
                        },
                    }
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}