use std::sync::{Arc, RwLock};
use std::time::Duration;
use ratatui::widgets::{TableState, ListState};
use std::collections::HashMap;
use bollard::models::ContainerInspectResponse;
use futures::StreamExt;

use crate::docker::client::DockerClient;
use crate::types::{ContainerInfo, ContainerStats, Result};
use crate::docker::containers::{list_containers, start_container, stop_container, restart_container, remove_container, inspect_container};
use crate::docker::logs::stream_logs;
use crate::docker::stats::fetch_container_stats;

pub struct App {
    pub docker: DockerClient,
    pub containers: Arc<RwLock<Vec<ContainerInfo>>>,
    pub container_stats: Arc<RwLock<HashMap<String, ContainerStats>>>,
    pub table_state: TableState,
    pub show_all: bool,
    
    // Selection state
    pub selected_container_details: Arc<RwLock<Option<String>>>,
    pub selected_container_logs: Arc<RwLock<Vec<String>>>,
    pub last_fetched_id: Option<String>,
    
    // Logs state
    pub logs_state: ListState,
    pub auto_scroll: bool,
    pub log_stream_task: Option<tokio::task::JoinHandle<()>>,

    // Metrics
    pub total_containers: usize,
    pub running_count: usize,
    pub stopped_count: usize,
    pub paused_count: usize,

    // UI State
    pub show_help: bool,
}

impl App {
    pub async fn new() -> Result<Self> {
        let docker = DockerClient::new()?;
        let containers = Arc::new(RwLock::new(Vec::new()));
        let container_stats = Arc::new(RwLock::new(HashMap::new()));
        
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
            if let Some(container) = app.selected_container() {
                 app.trigger_fetch(container.id);
            }
        }
        
        // Spawn background task for stats updates
        let docker_clone = app.docker.clone();
        let containers_clone = containers.clone();
        let stats_clone = container_stats.clone();
        
        tokio::spawn(async move {
            loop {
                let running_containers: Vec<String> = {
                    let containers = containers_clone.read().unwrap();
                    containers
                        .iter()
                        .filter(|c| c.state == "running")
                        .map(|c| c.id.clone())
                        .collect()
                };
                
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
                            match fetch_container_stats(&docker, &id).await {
                                Ok(Some((cpu, mem, limit))) => Some((id, cpu, mem, limit)),
                                _ => None,
                            }
                        }
                    })
                    .collect();
                
                let results = futures::future::join_all(stats_futures).await;
                
                {
                    let mut stats_map = stats_clone.write().unwrap();
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
                }
                
                tokio::time::sleep(Duration::from_secs(3)).await;
            }
        });
        
        Ok(app)
    }

    pub async fn refresh_containers(&mut self) -> Result<()> {
        let containers_result = list_containers(&self.docker, self.show_all).await?;

        self.total_containers = containers_result.len();
        self.running_count = 0;
        self.stopped_count = 0;
        self.paused_count = 0;

        for c in &containers_result {
             match c.state.as_str() {
                "running" => self.running_count += 1,
                "exited" => self.stopped_count += 1,
                "paused" => self.paused_count += 1,
                _ => {}
            }
        }

        let mut containers = self.containers.write().unwrap();
        *containers = containers_result;
        drop(containers);
        Ok(())
    }

    pub fn next(&mut self) {
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

    pub fn previous(&mut self) {
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

    pub fn selected_container(&self) -> Option<ContainerInfo> {
        let containers = self.containers.read().unwrap();
        self.table_state
            .selected()
            .and_then(|i| containers.get(i).cloned())
    }

    pub fn trigger_fetch(&mut self, container_id: String) {
        if self.last_fetched_id.as_ref() == Some(&container_id) {
            return;
        }
        
        self.last_fetched_id = Some(container_id.clone());
        
        // Clear previous data
        {
            let mut details = self.selected_container_details.write().unwrap();
            *details = None;
            let mut logs = self.selected_container_logs.write().unwrap();
            logs.clear();
        }

        let docker = self.docker.clone();
        let details_lock = self.selected_container_details.clone();
        let id_clone = container_id.clone();

        // Spawn details fetch
        tokio::spawn(async move {
            let details_res = inspect_container(&docker, &id_clone).await;
            let details_str = match details_res {
                Ok(info) => format_details(info),
                Err(e) => format!("Error fetching details: {}", e),
            };
            *details_lock.write().unwrap() = Some(details_str);
        });

        // Start log stream
        self.start_log_stream(container_id);
    }

    fn start_log_stream(&mut self, container_id: String) {
        // Abort previous task
        if let Some(handle) = self.log_stream_task.take() {
            handle.abort();
        }

        let docker = self.docker.clone();
        let logs_lock = self.selected_container_logs.clone();
        
        let task = tokio::spawn(async move {
            let mut stream = stream_logs(&docker, &container_id, "100");
            
            while let Some(log_result) = stream.next().await {
                match log_result {
                    Ok(log) => {
                        let mut logs = logs_lock.write().unwrap();
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

    pub async fn restart_container(&mut self) -> Result<()> {
        if let Some(container) = self.selected_container() {
            restart_container(&self.docker, &container.id).await?;
        }
        Ok(())
    }

    pub async fn stop_container(&mut self) -> Result<()> {
        if let Some(container) = self.selected_container() {
            stop_container(&self.docker, &container.id).await?;
        }
        Ok(())
    }

    pub async fn start_container(&mut self) -> Result<()> {
        if let Some(container) = self.selected_container() {
            start_container(&self.docker, &container.id).await?;
        }
        Ok(())
    }

    pub async fn remove_container(&mut self) -> Result<()> {
        if let Some(container) = self.selected_container() {
            remove_container(&self.docker, &container.id).await?;
            self.refresh_containers().await?;
            // Reset selection if out of bounds
            if self.total_containers > 0 && self.table_state.selected().unwrap_or(0) >= self.total_containers {
                 self.table_state.select(Some(self.total_containers - 1));
            }
        }
        Ok(())
    }

    pub fn toggle_filter(&mut self) {
        self.show_all = !self.show_all;
    }
}

// Helper functions moved from main.rs
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
