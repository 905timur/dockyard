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
use bollard::container::{ListContainersOptions, LogsOptions, StatsOptions, InspectContainerOptions};
use bollard::models::ContainerInspectResponse;
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
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, List, ListItem, ListState, Paragraph, Row, Sparkline, Table, TableState, Wrap},
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
    cpu_history: Vec<u64>,
    memory_history: Vec<u64>,
}

struct App {
    docker: Docker,
    containers: Arc<RwLock<Vec<ContainerInfo>>>,
    container_stats: Arc<RwLock<std::collections::HashMap<String, ContainerStats>>>,
    table_state: TableState,
    show_all: bool,
    
    // Selection state
    selected_container_details: Arc<RwLock<Option<String>>>,
    selected_container_logs: Arc<RwLock<Vec<String>>>,
    last_fetched_id: Option<String>,
    
    // Logs state
    logs_state: ListState,
    auto_scroll: bool,
    log_stream_task: Option<tokio::task::JoinHandle<()>>,

    // Metrics
    total_containers: usize,
    running_count: usize,
    stopped_count: usize,
    paused_count: usize,

    // UI State
    show_help: bool,
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
            selected_container_details: Arc::new(RwLock::new(None)),
            selected_container_logs: Arc::new(RwLock::new(Vec::new())),
            last_fetched_id: None,
            logs_state: ListState::default(),
            auto_scroll: true,
            log_stream_task: None,
            total_containers: 0,
            running_count: 0,
            stopped_count: 0,
            paused_count: 0,
            show_help: false,
        };
        
        app.refresh_containers().await?;
        if app.total_containers > 0 {
            app.table_state.select(Some(0));
            // Trigger initial fetch
            if let Some(container) = app.selected_container().await {
                 app.trigger_fetch(container.id).await;
            }
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
                
                if running_containers.is_empty() {
                    tokio::time::sleep(Duration::from_secs(3)).await;
                    continue;
                }

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
                                    cpu_percent,
                                    memory_usage,
                                    memory_limit,
                                ))
                            } else {
                                None
                            }
                        }
                    })
                    .collect();
                
                let results = futures::future::join_all(stats_futures).await;
                
                let mut stats_map = stats_clone.write().await;
                for (id, cpu, mem, limit) in results.into_iter().flatten() {
                    stats_map.entry(id)
                        .and_modify(|stats| {
                            stats.cpu_percent = cpu;
                            stats.memory_usage = mem;
                            stats.memory_limit = limit;
                            stats.cpu_history.push((cpu * 100.0) as u64);
                            stats.memory_history.push(mem);
                            if stats.cpu_history.len() > 100 {
                                stats.cpu_history.remove(0);
                            }
                            if stats.memory_history.len() > 100 {
                                stats.memory_history.remove(0);
                            }
                        })
                        .or_insert_with(|| ContainerStats {
                            cpu_percent: cpu,
                            memory_usage: mem,
                            memory_limit: limit,
                            cpu_history: vec![(cpu * 100.0) as u64],
                            memory_history: vec![mem],
                        });
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

    async fn trigger_fetch(&mut self, container_id: String) {
        if self.last_fetched_id.as_ref() == Some(&container_id) {
            return;
        }
        
        self.last_fetched_id = Some(container_id.clone());
        
        // Clear previous data
        {
            let mut details = self.selected_container_details.write().await;
            *details = None;
            let mut logs = self.selected_container_logs.write().await;
            logs.clear();
        }

        let docker = self.docker.clone();
        let details_lock = self.selected_container_details.clone();
        let id_clone = container_id.clone();

        // Spawn details fetch
        tokio::spawn(async move {
            let details_res = docker.inspect_container(&id_clone, None::<InspectContainerOptions>).await;
            let details_str = match details_res {
                Ok(info) => format_details(info),
                Err(e) => format!("Error fetching details: {}", e),
            };
            *details_lock.write().await = Some(details_str);
        });

        // Start log stream
        self.start_log_stream(container_id).await;
    }

    async fn start_log_stream(&mut self, container_id: String) {
        // Abort previous task
        if let Some(handle) = self.log_stream_task.take() {
            handle.abort();
        }

        let docker = self.docker.clone();
        let logs_lock = self.selected_container_logs.clone();
        
        let task = tokio::spawn(async move {
            let options = LogsOptions::<String> {
                stdout: true,
                stderr: true,
                follow: true,
                tail: "100".to_string(),
                timestamps: true,
                ..Default::default()
            };
            
            let mut stream = docker.logs(&container_id, Some(options));
            
            while let Some(log_result) = stream.next().await {
                match log_result {
                    Ok(log) => {
                        let mut logs = logs_lock.write().await;
                        logs.push(log.to_string());
                        // Keep last 1000 lines to prevent memory issues
                        if logs.len() > 1000 {
                            logs.remove(0);
                        }
                    }
                    Err(_) => break,
                }
            }
        });
        
        self.log_stream_task = Some(task);
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
            // Reset selection if out of bounds
            if self.total_containers > 0 && self.table_state.selected().unwrap_or(0) >= self.total_containers {
                 self.table_state.select(Some(self.total_containers - 1));
            }
        }
        Ok(())
    }

    fn toggle_filter(&mut self) {
        self.show_all = !self.show_all;
    }
}

fn format_details(info: ContainerInspectResponse) -> String {
    let mut s = String::new();
    
    // Image & Name
    s.push_str("NAME: ");
    s.push_str(&info.name.unwrap_or_default().trim_start_matches('/').to_string());
    s.push_str("\n\n");

    s.push_str("IMAGE: ");
    s.push_str(&info.config.as_ref().and_then(|c| c.image.clone()).unwrap_or_default());
    s.push_str("\n\n");

    // Network
    s.push_str("NETWORK:\n");
    if let Some(net) = info.network_settings {
        if let Some(ports) = net.ports {
            for (k, v) in ports {
                if let Some(bindings) = v {
                    for b in bindings {
                        s.push_str(&format!("  {} -> {}:{}\n", k, b.host_ip.clone().unwrap_or_default(), b.host_port.clone().unwrap_or_default()));
                    }
                } else {
                    s.push_str(&format!("  {}\n", k));
                }
            }
        }
        if let Some(networks) = net.networks {
            for (name, _) in networks {
                s.push_str(&format!("  Network: {}\n", name));
            }
        }
    }
    s.push('\n');

    // Resources
    s.push_str("RESOURCES:\n");
    if let Some(host_config) = info.host_config.as_ref() {
        s.push_str(&format!("  Memory: {}\n", format_bytes(host_config.memory.unwrap_or(0) as u64)));
        s.push_str(&format!("  NanoCPUs: {}\n", host_config.nano_cpus.unwrap_or(0)));
        if let Some(restart) = &host_config.restart_policy {
            let restart_policy = restart.name.as_ref()
                .map(|n| format!("{:?}", n))
                .unwrap_or_else(|| "no".to_string());
            s.push_str(&format!("  Restart: {}\n", restart_policy));
        }
    }
    s.push('\n');

    // Environment
    s.push_str("ENV:\n");
    if let Some(config) = info.config {
        if let Some(env) = config.env {
            for e in env {
                s.push_str(&format!("  {}\n", e));
            }
        }
    }
    s.push('\n');

    // Created
    s.push_str(&format!("Created: {}\n", info.created.unwrap_or_default()));

    s
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

async fn draw_ui(f: &mut Frame<'_>, app: &App) {
    let area = f.area();
    
    // Split screen: Left (25%) and Right (75%)
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(75),
        ])
        .split(area);
    
    let left_pane = main_chunks[0];
    let right_pane = main_chunks[1];

    // Split Right Pane: Top (50%) and Bottom (50%)
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(right_pane);

    let top_right_pane = right_chunks[0];
    let bottom_right_pane = right_chunks[1];

    // Render Panes
    render_container_details(f, left_pane, app).await;
    render_container_list(f, top_right_pane, app).await;
    render_container_logs(f, bottom_right_pane, app).await;

    if app.show_help {
        render_help(f, area);
    }
}

fn render_help(f: &mut Frame<'_>, area: Rect) {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(area);

    let popup_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(popup_layout[1])[1];

    let help_text = vec![
        Line::from("Navigation:"),
        Line::from("  ↑/↓ or j/k: Select container"),
        Line::from("  Esc or q: Close help / Quit"),
        Line::from(""),
        Line::from("Actions:"),
        Line::from("  s: Stop container"),
        Line::from("  t: Start container"),
        Line::from("  r: Restart container"),
        Line::from("  d: Remove container"),
        Line::from("  f: Toggle filter (All/Running)"),
        Line::from("  a: Toggle auto-scroll logs"),
        Line::from("  J/K: Scroll logs manually"),
        Line::from(""),
        Line::from("General:"),
        Line::from("  ?: Toggle this help menu"),
    ];

    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(help_text)
        .block(block)
        .wrap(Wrap { trim: true });

    f.render_widget(Clear, popup_area);
    f.render_widget(paragraph, popup_area);
}

async fn render_container_details(f: &mut Frame<'_>, area: Rect, app: &App) {
    let details_lock = app.selected_container_details.read().await;
    let details_text = match details_lock.as_ref() {
        Some(text) => text.clone(),
        None => "Select a container to view details".to_string(),
    };
    drop(details_lock);

    // Split area: Top for text, Bottom for graphs
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10), // Text area
            Constraint::Length(10), // Graphs area
        ])
        .split(area);

    let text_area = chunks[0];
    let graphs_area = chunks[1];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Details ")
        .border_style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(details_text)
        .block(block)
        .wrap(Wrap { trim: true });
    
    f.render_widget(paragraph, text_area);

    // Render Graphs if a container is selected
    if let Some(container) = app.selected_container().await {
        let stats_map = app.container_stats.read().await;
        if let Some(stats) = stats_map.get(&container.id) {
            // Split graphs area: Left CPU, Right Memory
            let graph_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(50),
                    Constraint::Percentage(50),
                ])
                .split(graphs_area);
            
            // CPU Graph
            let cpu_block = Block::default()
                .borders(Borders::ALL)
                .title(format!(" CPU: {:.2}% ", stats.cpu_percent));
            
            let cpu_sparkline = Sparkline::default()
                .block(cpu_block)
                .data(&stats.cpu_history)
                .style(Style::default().fg(Color::Green));
            
            f.render_widget(cpu_sparkline, graph_chunks[0]);

            // Memory Graph
            let mem_block = Block::default()
                .borders(Borders::ALL)
                .title(format!(" MEM: {} ", format_bytes(stats.memory_usage)));
            
            let mem_sparkline = Sparkline::default()
                .block(mem_block)
                .data(&stats.memory_history)
                .style(Style::default().fg(Color::Magenta));
             
            f.render_widget(mem_sparkline, graph_chunks[1]);
        }
    }
}

async fn render_container_list(f: &mut Frame<'_>, area: Rect, app: &App) {
    let containers = app.containers.read().await;
    
    // Header cells - simplified for compact view if needed, but we have space
    let header_cells = ["NAME", "STATUS", "IMG", "UP"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Black).bg(Color::Cyan).bold()));
    let header = Row::new(header_cells).height(1);

    let rows = containers.iter().map(|c| {
        let (status_symbol, status_color) = match c.state.as_str() {
            "running" => ("●", Color::Green),
            "exited" => ("■", Color::Red),
            "paused" => ("‖", Color::Yellow),
            _ => ("○", Color::Gray),
        };

        let uptime = if c.state == "running" {
            format_uptime(c.created)
        } else {
            "-".to_string()
        };

        // Shorten image name
        let image = if c.image.len() > 15 {
             format!("{}...", &c.image[0..12])
        } else {
             c.image.clone()
        };

        let cells = vec![
            Cell::from(c.name.clone()).style(Style::default().fg(Color::Cyan)),
            Cell::from(format!("{} {}", status_symbol, c.state))
                .style(Style::default().fg(status_color).bold()),
            Cell::from(image),
            Cell::from(uptime),
        ];
        Row::new(cells).height(1)
    });

    // Adjust constraints for the list columns
    let widths = [
        Constraint::Percentage(30),
        Constraint::Percentage(20),
        Constraint::Percentage(30),
        Constraint::Percentage(20),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Containers ({}) ", app.total_containers))
                .border_style(Style::default().fg(Color::Magenta))
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    f.render_stateful_widget(table, area, &mut app.table_state.clone());
}

async fn render_container_logs(f: &mut Frame<'_>, area: Rect, app: &App) {
    let logs_lock = app.selected_container_logs.read().await;
    
    let logs_items: Vec<ListItem> = logs_lock
        .iter()
        .map(|log| {
             let lower = log.to_lowercase();
             let style = if lower.contains("error") {
                 Style::default().fg(Color::Red)
             } else if lower.contains("warn") {
                 Style::default().fg(Color::Yellow)
             } else if lower.contains("info") {
                 Style::default().fg(Color::Green)
             } else {
                 Style::default().fg(Color::White)
             };
             ListItem::new(Line::from(Span::styled(log.as_str(), style)))
        })
        .collect();

    let title = if app.auto_scroll {
        " Logs (Live - Auto Scroll) "
    } else {
        " Logs (Live - Manual Scroll) "
    };

    let logs_list = List::new(logs_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::Yellow))
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    let mut state = app.logs_state.clone();
    f.render_stateful_widget(logs_list, area, &mut state);
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
    let mut last_selection_change = Instant::now();
    let mut needs_fetch = true; // Fetch initially

    loop {
        // Refresh container list every 5 seconds
        if last_container_update.elapsed() > Duration::from_secs(5) {
            let _ = app.refresh_containers().await;
            last_container_update = Instant::now();
        }

        // Debounced Fetch
        if needs_fetch && last_selection_change.elapsed() > Duration::from_millis(150) {
            if let Some(container) = app.selected_container().await {
                app.trigger_fetch(container.id).await;
            } else {
                 // Clear if nothing selected
                *app.selected_container_details.write().await = None;
                app.selected_container_logs.write().await.clear();
            }
            needs_fetch = false;
        }

        // Auto-scroll logs
        if app.auto_scroll {
            let logs_len = app.selected_container_logs.read().await.len();
            if logs_len > 0 {
                app.logs_state.select(Some(logs_len - 1));
            }
        }

        // Draw UI
        terminal.draw(|f| {
            let app_ref = &app;
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(draw_ui(f, app_ref))
            });
        })?;

        // Poll for events
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if app.show_help {
                        match key.code {
                            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {
                                app.show_help = false;
                            }
                            _ => {}
                        }
                    } else {
                        match key.code {
                            KeyCode::Char('?') => app.show_help = true,
                            KeyCode::Char('q') | KeyCode::Esc => break,
                            KeyCode::Down | KeyCode::Char('j') => {
                                app.next();
                                last_selection_change = Instant::now();
                                needs_fetch = true;
                            },
                            KeyCode::Up | KeyCode::Char('k') => {
                                app.previous();
                                last_selection_change = Instant::now();
                                needs_fetch = true;
                            },
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
                                needs_fetch = true;
                            }
                            KeyCode::Char('a') => {
                                app.auto_scroll = !app.auto_scroll;
                            }
                            KeyCode::Char('J') => {
                                app.auto_scroll = false;
                                let logs_len = app.selected_container_logs.read().await.len();
                                if logs_len > 0 {
                                    let i = match app.logs_state.selected() {
                                        Some(i) => {
                                            if i >= logs_len - 1 { logs_len - 1 } else { i + 1 }
                                        },
                                        None => 0,
                                    };
                                    app.logs_state.select(Some(i));
                                }
                            }
                            KeyCode::Char('K') => {
                                app.auto_scroll = false;
                                let logs_len = app.selected_container_logs.read().await.len();
                                if logs_len > 0 {
                                    let i = match app.logs_state.selected() {
                                        Some(i) => {
                                            if i == 0 { 0 } else { i - 1 }
                                        },
                                        None => 0,
                                    };
                                    app.logs_state.select(Some(i));
                                }
                            }
                            _ => {}
                        }
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
