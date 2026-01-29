use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use ratatui::widgets::{TableState, ListState};
use std::collections::HashMap;
use bollard::models::ContainerInspectResponse;
use futures::StreamExt;
use tokio::sync::Semaphore;
use chrono::Utc;

use crate::config::{load_config, save_config};
use crate::docker::client::DockerClient;
use crate::types::{ContainerInfo, ContainerStats, ImageInfo, Result, ContainerHealth, HealthStatus, AppConfig, RefreshRate, StatsView, PollStrategy, PerfMetrics};
use crate::docker::containers::{list_containers, start_container, stop_container, restart_container, remove_container, inspect_container, pause_container, unpause_container};
use sysinfo::{Pid, System};
use crate::docker::health::{fetch_health_info, parse_health_status_from_string};
use crate::docker::images::{list_images, pull_image, remove_image, inspect_image, prune_images};
use crate::docker::logs::stream_logs;
use crate::docker::stats::fetch_container_stats;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    ContainerList,
    Logs,
}

#[derive(Debug, Clone, PartialEq)]
pub enum View {
    Containers,
    Images,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SortOrder {
    CreatedDesc,
    CreatedAsc,
    SizeDesc,
    SizeAsc,
    HealthDesc, // Unhealthy first
    HealthAsc,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HealthFilter {
    All,
    Unhealthy,
    Healthy,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct ViewportState {
    pub offset: usize,
    pub height: u16,
}

pub struct App {
    pub docker: DockerClient,
    pub containers: Arc<RwLock<Vec<ContainerInfo>>>,
    pub filtered_containers: Vec<ContainerInfo>, // Cache for UI
    pub container_stats: Arc<RwLock<HashMap<String, ContainerStats>>>,
    pub container_health: Arc<RwLock<HashMap<String, ContainerHealth>>>,
    pub perf_metrics: Arc<RwLock<PerfMetrics>>,
    pub table_state: TableState,
    pub viewport_state: Arc<RwLock<ViewportState>>,
    pub config: Arc<RwLock<AppConfig>>,
    pub show_all: Arc<AtomicBool>,
    pub health_filter: HealthFilter,
    pub container_sort: SortOrder,
    
    // Image State
    pub images: Arc<RwLock<Vec<ImageInfo>>>,
    pub table_state_images: TableState,
    pub current_view: View,
    pub show_dangling: Arc<AtomicBool>,
    pub total_images: usize,
    pub total_image_size: u64,
    pub image_sort: SortOrder,
    pub selected_image_details: Arc<RwLock<Option<String>>>,
    
    // Pull Image State
    pub show_pull_dialog: bool,
    pub pull_input: String,
    pub is_pulling: Arc<AtomicBool>,
    pub show_health_log_dialog: bool,
    pub health_log_content: String,
    pub pull_progress: Arc<RwLock<Vec<String>>>, // Store recent progress lines
    pub show_delete_confirm: bool, // For image deletion
    pub pending_delete_force: bool,

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
    pub current_help_tab: crate::types::HelpTab,
    pub help_scroll: u16,
    pub should_exec: Option<String>,
    pub focus: Focus,
}

impl App {
    pub async fn new(_stats_interval_arg: u64) -> Result<Self> {
        let docker = DockerClient::new()?;
        let config = load_config().unwrap_or_default();
        let containers = Arc::new(RwLock::new(Vec::new()));
        let container_stats = Arc::new(RwLock::new(HashMap::new()));
        let container_health = Arc::new(RwLock::new(HashMap::new()));
        let perf_metrics = Arc::new(RwLock::new(PerfMetrics::default()));
        let viewport_state = Arc::new(RwLock::new(ViewportState::default()));
        
        let mut app = Self {
            docker,
            containers: containers.clone(),
            filtered_containers: Vec::new(),
            container_stats: container_stats.clone(),
            container_health: container_health.clone(),
            perf_metrics: perf_metrics.clone(),
            table_state: TableState::default(),
            viewport_state: viewport_state.clone(),
            config: Arc::new(RwLock::new(config)),
            show_all: Arc::new(AtomicBool::new(true)),
            health_filter: HealthFilter::All,
            container_sort: SortOrder::CreatedDesc,
            
            // Image init
            images: Arc::new(RwLock::new(Vec::new())),
            table_state_images: TableState::default(),
            current_view: View::Containers,
            show_dangling: Arc::new(AtomicBool::new(false)),
            total_images: 0,
            total_image_size: 0,
            image_sort: SortOrder::CreatedDesc,
            selected_image_details: Arc::new(RwLock::new(None)),
            show_pull_dialog: false,
            pull_input: String::new(),
            is_pulling: Arc::new(AtomicBool::new(false)),
            show_health_log_dialog: false,
            health_log_content: String::new(),
            pull_progress: Arc::new(RwLock::new(Vec::new())),
            show_delete_confirm: false,
            pending_delete_force: false,

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
            current_help_tab: crate::types::HelpTab::default(),
            help_scroll: 0,
            should_exec: None,
            focus: Focus::ContainerList,
        };
        
        app.refresh_containers().await?;
        app.refresh_images().await?;
        if app.total_containers > 0 {
            app.table_state.select(Some(0));
            // Trigger initial fetch
            if let Some(container) = app.selected_container() {
                 app.trigger_fetch(container.id);
            }
        }
        
        // --- Background Task 1: List Containers (every 10s) ---
        let docker_clone_list = app.docker.clone();
        let containers_clone_list = containers.clone();
        let show_all_clone = app.show_all.clone();
        let health_map_list = container_health.clone();
        let docker_health_list = app.docker.clone();
        
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(10)).await;
                let show_all = show_all_clone.load(Ordering::Relaxed);
                match list_containers(&docker_clone_list, show_all).await {
                    Ok(containers_result) => {
                         // Check for health changes
                         {
                             let health_map = health_map_list.write().unwrap();
                             for c in &containers_result {
                                 if c.state != "running" { continue; }
                                 
                                 let new_status = parse_health_status_from_string(&c.status);
                                 let needs_update = match health_map.get(&c.id) {
                                     Some(current) => current.status != new_status,
                                     None => true,
                                 };

                                 if needs_update {
                                     let docker = docker_health_list.clone();
                                     let health_map_inner = health_map_list.clone();
                                     let id = c.id.clone();
                                     tokio::spawn(async move {
                                         if let Ok(health) = fetch_health_info(&docker, &id).await {
                                             health_map_inner.write().unwrap().insert(id, health);
                                         }
                                     });
                                 }
                             }
                         }

                         let mut containers = containers_clone_list.write().unwrap();
                         *containers = containers_result;
                    }
                    Err(e) => {
                        eprintln!("Failed to refresh containers: {}", e);
                    }
                }
            }
        });

        // --- Background Task 3: Health Monitoring (Events & Polling) ---
        let docker_events = app.docker.clone();
        let health_map_events = container_health.clone();
        
        tokio::spawn(async move {
            use bollard::system::EventsOptions;
            let mut filters = HashMap::new();
            filters.insert("type".to_string(), vec!["container".to_string()]);
            filters.insert("event".to_string(), vec!["health_status".to_string()]);
            
            let options = EventsOptions {
                filters,
                ..Default::default()
            };
            
            let mut stream = docker_events.inner.events(Some(options));
            
            while let Some(event_res) = stream.next().await {
                 if let Ok(event) = event_res {
                     if let Some(actor) = event.actor {
                         if let Some(id) = actor.id {
                             let id = id.to_string();
                             let docker = docker_events.clone();
                             let health_map = health_map_events.clone();
                             tokio::spawn(async move {
                                 if let Ok(health) = fetch_health_info(&docker, &id).await {
                                     health_map.write().unwrap().insert(id, health);
                                 }
                             });
                         }
                     }
                 }
            }
        });

        // Periodic Polling for Unhealthy containers (every 5s)
        let docker_poll = app.docker.clone();
        let health_map_poll = container_health.clone();
        
        tokio::spawn(async move {
             loop {
                 tokio::time::sleep(Duration::from_secs(5)).await;
                 
                 let ids_to_check: Vec<String> = {
                     let map = health_map_poll.read().unwrap();
                     map.iter()
                        .filter(|(_, h)| h.status == HealthStatus::Unhealthy || h.status == HealthStatus::Starting)
                        .map(|(id, _)| id.clone())
                        .collect()
                 };

                 for id in ids_to_check {
                     let docker = docker_poll.clone();
                     let map = health_map_poll.clone();
                     tokio::spawn(async move {
                         if let Ok(health) = fetch_health_info(&docker, &id).await {
                             map.write().unwrap().insert(id, health);
                         }
                     });
                 }
             }
        });

        // --- Background Task 1.5: List Images (every 30s) ---
        let docker_clone_images = app.docker.clone();
        let images_clone = app.images.clone();
        let show_dangling_clone = app.show_dangling.clone();

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(30)).await;
                let show_dangling = show_dangling_clone.load(Ordering::Relaxed);
                match list_images(&docker_clone_images, show_dangling).await {
                    Ok(images_result) => {
                        let mut images = images_clone.write().unwrap();
                        *images = images_result;
                    }
                    Err(e) => {
                        eprintln!("Failed to refresh images: {}", e);
                    }
                }
            }
        });
        
        // --- Background Task 4: Performance Monitoring ---
        let perf_metrics_clone = perf_metrics.clone();
        
        std::thread::spawn(move || {
            let mut sys = System::new();
            let pid = Pid::from(std::process::id() as usize);
            
            loop {
                sys.refresh_process(pid);
                if let Some(process) = sys.process(pid) {
                    let cpu = process.cpu_usage();
                    let mem = process.memory();
                    
                    if let Ok(mut metrics) = perf_metrics_clone.write() {
                        metrics.cpu_usage = cpu as f64;
                        metrics.memory_usage = mem;
                    }
                }
                std::thread::sleep(Duration::from_secs(2));
            }
        });

        // --- Background Task 2: Fetch Stats (Dynamic Polling) ---
        let docker_clone = app.docker.clone();
        let containers_clone = containers.clone();
        let stats_clone = container_stats.clone();
        let viewport_clone = viewport_state.clone();
        let config_clone = app.config.clone();
        let perf_metrics_poll = app.perf_metrics.clone();
        
        tokio::spawn(async move {
            let semaphore = Arc::new(Semaphore::new(10));

            loop {
                let start_time = tokio::time::Instant::now();
                
                let (refresh_rate, poll_strategy, viewport_buffer) = {
                    let c = config_clone.read().unwrap();
                    (c.refresh_rate.clone(), c.poll_strategy.clone(), c.viewport_buffer)
                };

                let interval_ms = match refresh_rate {
                    RefreshRate::Manual => {
                         tokio::time::sleep(Duration::from_millis(500)).await;
                         continue;
                    },
                    RefreshRate::Interval(d) => d.as_millis() as u64,
                };

                let targets: Vec<String> = {
                    let containers = containers_clone.read().unwrap();
                    let total = containers.len();
                    
                    if total == 0 {
                        Vec::new()
                    } else {
                        match poll_strategy {
                            PollStrategy::AllContainers => {
                                containers.iter()
                                    .filter(|c| c.state == "running")
                                    .map(|c| c.id.clone())
                                    .collect()
                            },
                            PollStrategy::VisibleOnly => {
                                let viewport = viewport_clone.read().unwrap();
                                let start = viewport.offset.saturating_sub(viewport_buffer);
                                let end = (viewport.offset + viewport.height as usize + viewport_buffer).min(total);
                                
                                if start >= total {
                                    Vec::new()
                                } else {
                                    let actual_end = end.min(total);
                                    containers[start..actual_end]
                                        .iter()
                                        .filter(|c| c.state == "running")
                                        .map(|c| c.id.clone())
                                        .collect()
                                }
                            }
                        }
                    }
                };

                if targets.is_empty() {
                    tokio::time::sleep(Duration::from_millis(1000)).await;
                    continue;
                }

                let target_count = targets.len();
                let delay_per_req = if target_count > 0 {
                    interval_ms / (target_count as u64).max(1)
                } else {
                    0
                };

                let mut tasks = Vec::new();

                for (i, id) in targets.into_iter().enumerate() {
                    let docker = docker_clone.clone();
                    let stats_map = stats_clone.clone();
                    let sem = semaphore.clone();
                    
                    let delay = std::cmp::min(delay_per_req * i as u64, interval_ms);

                    tasks.push(tokio::spawn(async move {
                        tokio::time::sleep(Duration::from_millis(delay)).await;
                        
                        let _permit = sem.acquire().await.unwrap();
                        
                        match fetch_container_stats(&docker, &id).await {
                            Ok(Some((cpu, user_cpu, system_cpu, mem, cached_mem, limit))) => {
                                let mut map = stats_map.write().unwrap();
                                let now = Utc::now().timestamp();
                                map.entry(id)
                                    .and_modify(|stats| {
                                        stats.cpu_percent = cpu;
                                        stats.user_cpu_percent = user_cpu;
                                        stats.system_cpu_percent = system_cpu;
                                        stats.memory_usage = mem;
                                        stats.cached_memory = cached_mem;
                                        stats.memory_limit = limit;
                                        stats.last_updated = now;
                                        stats.cpu_history.push((cpu * 100.0) as u64);
                                        stats.user_cpu_history.push((user_cpu * 100.0) as u64);
                                        stats.system_cpu_history.push((system_cpu * 100.0) as u64);
                                        stats.memory_history.push(mem);
                                        stats.cached_memory_history.push(cached_mem);
                                        if stats.cpu_history.len() > 100 {
                                            stats.cpu_history.remove(0);
                                        }
                                        if stats.user_cpu_history.len() > 100 {
                                            stats.user_cpu_history.remove(0);
                                        }
                                        if stats.system_cpu_history.len() > 100 {
                                            stats.system_cpu_history.remove(0);
                                        }
                                        if stats.memory_history.len() > 100 {
                                            stats.memory_history.remove(0);
                                        }
                                        if stats.cached_memory_history.len() > 100 {
                                            stats.cached_memory_history.remove(0);
                                        }
                                    })
                                    .or_insert_with(|| ContainerStats {
                                        cpu_percent: cpu,
                                        user_cpu_percent: user_cpu,
                                        system_cpu_percent: system_cpu,
                                        memory_usage: mem,
                                        cached_memory: cached_mem,
                                        memory_limit: limit,
                                        cpu_history: vec![(cpu * 100.0) as u64],
                                        user_cpu_history: vec![(user_cpu * 100.0) as u64],
                                        system_cpu_history: vec![(system_cpu * 100.0) as u64],
                                        memory_history: vec![mem],
                                        cached_memory_history: vec![cached_mem],
                                        last_updated: now,
                                    });
                            }
                            Ok(None) => {}
                            Err(e) => {
                                eprintln!("Failed to fetch stats for {}: {}", id, e);
                            }
                        }
                    }));
                }
                
                let elapsed = start_time.elapsed();
                
                if let Ok(mut metrics) = perf_metrics_poll.write() {
                    metrics.poll_time_ms = elapsed.as_millis() as u64;
                }

                if elapsed < Duration::from_millis(interval_ms) {
                    tokio::time::sleep(Duration::from_millis(interval_ms) - elapsed).await;
                }
            }
        });
        
        Ok(app)
    }

    pub async fn refresh_containers(&mut self) -> Result<()> {
        let containers_result = list_containers(&self.docker, self.show_all.load(Ordering::Relaxed)).await?;

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
        
        self.update_filtered_containers();
        Ok(())
    }

    pub fn update_filtered_containers(&mut self) {
        let containers = self.containers.read().unwrap();
        let health = self.container_health.read().unwrap();
        
        let mut filtered: Vec<ContainerInfo> = containers.iter().filter(|c| {
             match self.health_filter {
                 HealthFilter::All => true,
                 HealthFilter::Unhealthy => {
                      if let Some(h) = health.get(&c.id) {
                          h.status == HealthStatus::Unhealthy || h.status == HealthStatus::Starting
                      } else {
                          false
                      }
                 },
                 HealthFilter::Healthy => {
                      if let Some(h) = health.get(&c.id) {
                          h.status == HealthStatus::Healthy
                      } else {
                          false
                      }
                 }
             }
        }).cloned().collect();
        
        match self.container_sort {
            SortOrder::CreatedDesc => filtered.sort_by(|a, b| b.created.cmp(&a.created)),
            SortOrder::CreatedAsc => filtered.sort_by(|a, b| a.created.cmp(&b.created)),
            SortOrder::HealthDesc => {
                filtered.sort_by(|a, b| {
                    let ha = health.get(&a.id).map(|h| &h.status).unwrap_or(&HealthStatus::NoHealthCheck);
                    let hb = health.get(&b.id).map(|h| &h.status).unwrap_or(&HealthStatus::NoHealthCheck);
                    ha.cmp(hb)
                });
            },
            SortOrder::HealthAsc => {
                filtered.sort_by(|a, b| {
                    let ha = health.get(&a.id).map(|h| &h.status).unwrap_or(&HealthStatus::NoHealthCheck);
                    let hb = health.get(&b.id).map(|h| &h.status).unwrap_or(&HealthStatus::NoHealthCheck);
                    ha.cmp(hb)
                });
            }
            _ => {
                 filtered.sort_by(|a, b| b.created.cmp(&a.created));
            }
        }

        self.filtered_containers = filtered;
        self.total_containers = self.filtered_containers.len();

        if self.total_containers > 0 {
             if let Some(selected) = self.table_state.selected() {
                 if selected >= self.total_containers {
                     self.table_state.select(Some(self.total_containers - 1));
                 }
             } else {
                 self.table_state.select(Some(0));
             }
        } else {
            self.table_state.select(None);
        }
    }

    pub fn cycle_container_sort(&mut self) {
        self.container_sort = match self.container_sort {
            SortOrder::CreatedDesc => SortOrder::CreatedAsc,
            SortOrder::CreatedAsc => SortOrder::HealthAsc,
            SortOrder::HealthAsc => SortOrder::CreatedDesc,
            _ => SortOrder::CreatedDesc,
        };
        self.update_filtered_containers();
    }

    pub fn toggle_health_filter(&mut self) {
        self.health_filter = match self.health_filter {
            HealthFilter::All => HealthFilter::Unhealthy,
            HealthFilter::Unhealthy => HealthFilter::Healthy,
            HealthFilter::Healthy => HealthFilter::All,
        };
        self.update_filtered_containers();
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
        self.table_state
            .selected()
            .and_then(|i| self.filtered_containers.get(i).cloned())
    }

    pub fn trigger_fetch(&mut self, container_id: String) {
        if self.last_fetched_id.as_ref() == Some(&container_id) {
            return;
        }
        
        self.last_fetched_id = Some(container_id.clone());
        
        {
            let mut details = self.selected_container_details.write().unwrap();
            *details = None;
            let mut logs = self.selected_container_logs.write().unwrap();
            logs.clear();
        }

        let docker = self.docker.clone();
        let details_lock = self.selected_container_details.clone();
        let id_clone = container_id.clone();

        tokio::spawn(async move {
            let details_res = inspect_container(&docker, &id_clone).await;
            let details_str = match details_res {
                Ok(info) => format_details(info),
                Err(e) => format!("Error fetching details: {}", e),
            };
            *details_lock.write().unwrap() = Some(details_str);
        });

        self.start_log_stream(container_id);
    }

    fn start_log_stream(&mut self, container_id: String) {
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
            if self.total_containers > 0 && self.table_state.selected().unwrap_or(0) >= self.total_containers {
                 self.table_state.select(Some(self.total_containers - 1));
            }
        }
        Ok(())
    }

    pub async fn pause_container(&mut self) -> Result<()> {
        if let Some(container) = self.selected_container() {
            if container.state == "running" {
                pause_container(&self.docker, &container.id).await?;
                self.refresh_containers().await?;
            }
        }
        Ok(())
    }

    pub async fn unpause_container(&mut self) -> Result<()> {
        if let Some(container) = self.selected_container() {
            if container.state == "paused" {
                unpause_container(&self.docker, &container.id).await?;
                self.refresh_containers().await?;
            }
        }
        Ok(())
    }

    pub async fn refresh_images(&mut self) -> Result<()> {
        let show_dangling = self.show_dangling.load(Ordering::Relaxed);
        let images_result = list_images(&self.docker, show_dangling).await?;
        
        self.total_images = images_result.len();
        self.total_image_size = images_result.iter().map(|i| i.size as u64).sum();

        let mut images = self.images.write().unwrap();
        *images = images_result;
        
        match self.image_sort {
            SortOrder::CreatedDesc => images.sort_by(|a, b| b.created.cmp(&a.created)),
            SortOrder::CreatedAsc => images.sort_by(|a, b| a.created.cmp(&b.created)),
            SortOrder::SizeDesc => images.sort_by(|a, b| b.size.cmp(&a.size)),
            SortOrder::SizeAsc => images.sort_by(|a, b| a.size.cmp(&b.size)),
            SortOrder::HealthDesc | SortOrder::HealthAsc => {
                images.sort_by(|a, b| b.created.cmp(&a.created));
            }
        }
        Ok(())
    }

    pub fn cycle_sort(&mut self) {
        self.image_sort = match self.image_sort {
            SortOrder::CreatedDesc => SortOrder::CreatedAsc,
            SortOrder::CreatedAsc => SortOrder::SizeDesc,
            SortOrder::SizeDesc => SortOrder::SizeAsc,
            SortOrder::SizeAsc => SortOrder::CreatedDesc,
            _ => SortOrder::CreatedDesc,
        };
    }

    pub fn next_image(&mut self) {
        if self.total_images == 0 { return; }
        let i = match self.table_state_images.selected() {
            Some(i) => if i >= self.total_images - 1 { 0 } else { i + 1 },
            None => 0,
        };
        self.table_state_images.select(Some(i));
    }

    pub fn previous_image(&mut self) {
        if self.total_images == 0 { return; }
        let i = match self.table_state_images.selected() {
            Some(i) => if i == 0 { self.total_images - 1 } else { i - 1 },
            None => 0,
        };
        self.table_state_images.select(Some(i));
    }

    pub fn selected_image(&self) -> Option<ImageInfo> {
        self.images.read().unwrap().get(self.table_state_images.selected()?).cloned()
    }

    pub fn trigger_image_details(&mut self) {
        if let Some(image) = self.selected_image() {
            let docker = self.docker.clone();
            let details_lock = self.selected_image_details.clone();
            tokio::spawn(async move {
                match inspect_image(&docker, &image.id).await {
                    Ok(info) => *details_lock.write().unwrap() = Some(format_image_details(info)),
                    Err(e) => *details_lock.write().unwrap() = Some(format!("Error: {}", e)),
                }
            });
        }
    }

    pub async fn remove_current_image(&mut self, force: bool) -> Result<()> {
        if let Some(image) = self.selected_image() {
            remove_image(&self.docker, &image.id, force).await?;
            self.refresh_images().await?;
        }
        Ok(())
    }

    pub async fn prune_images(&mut self) -> Result<()> {
        prune_images(&self.docker).await?;
        self.refresh_images().await?;
        Ok(())
    }

    pub fn start_pull_image(&mut self, image_name: String) {
        let docker = self.docker.clone();
        let progress_lock = self.pull_progress.clone();
        let is_pulling = self.is_pulling.clone();
        let images_ref = self.images.clone();
        
        is_pulling.store(true, Ordering::Relaxed);
        progress_lock.write().unwrap().clear();

        tokio::spawn(async move {
            let mut stream = pull_image(&docker, image_name);
            while let Some(res) = stream.next().await {
                let mut progress = progress_lock.write().unwrap();
                match res {
                    Ok(info) => {
                        let msg = format!("{:?}", info); // Simplest conversion
                        progress.push(msg);
                    }
                    Err(e) => progress.push(format!("Error: {}", e)),
                }
                if progress.len() > 100 { progress.remove(0); }
            }
            is_pulling.store(false, Ordering::Relaxed);
            
            if let Ok(imgs) = list_images(&docker, false).await {
                *images_ref.write().unwrap() = imgs;
            }
        });
    }

    pub fn toggle_filter(&mut self) {
        let current = self.show_all.load(Ordering::Relaxed);
        self.show_all.store(!current, Ordering::Relaxed);
    }

    pub fn apply_turbo_preset(&mut self) {
        let mut config = self.config.write().unwrap();
        if config.turbo_mode {
            config.refresh_rate = RefreshRate::Interval(Duration::from_secs(2));
            config.stats_view = StatsView::Minimal;
            config.poll_strategy = PollStrategy::VisibleOnly;
        } else {
            config.refresh_rate = RefreshRate::Interval(Duration::from_secs(1));
            config.stats_view = StatsView::Detailed;
            config.poll_strategy = PollStrategy::AllContainers;
        }
    }

    pub fn save_config(&self) {
        let config = self.config.read().unwrap();
        let _ = save_config(&config);
    }
}

pub fn format_details(info: ContainerInspectResponse) -> String {
    let mut s = String::new();
    s.push_str(&format!("ID: {}\n", info.id.as_deref().unwrap_or("Unknown")));
    s.push_str(&format!("Name: {}\n", info.name.as_deref().unwrap_or("Unknown")));
    s.push_str(&format!("Image: {}\n", info.image.as_deref().unwrap_or("Unknown")));
    s.push_str(&format!("Status: {}\n", info.state.as_ref().map(|st| format!("{:?}", st.status)).unwrap_or_else(|| "Unknown".to_string())));
    
    if let Some(config) = info.config {
        if let Some(env) = config.env {
            s.push_str("\nEnvironment:\n");
            for e in env { s.push_str(&format!("  {}\n", e)); }
        }
    }
    
    if let Some(mounts) = info.mounts {
        s.push_str("\nMounts:\n");
        for m in mounts {
            s.push_str(&format!("  {} -> {}\n", m.source.as_deref().unwrap_or("?"), m.destination.as_deref().unwrap_or("?")));
        }
    }
    
    s
}

pub fn format_image_details(info: bollard::models::ImageInspect) -> String {
    let mut s = String::new();
    s.push_str(&format!("ID: {}\n", info.id.as_deref().unwrap_or("Unknown")));
    if let Some(tags) = info.repo_tags {
        s.push_str("Tags:\n");
        for t in tags { s.push_str(&format!("  {}\n", t)); }
    }
    s.push_str(&format!("Size: {}\n", format_bytes(info.size.unwrap_or(0) as u64)));
    s
}

pub fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 { format!("{} B", bytes) }
    else if bytes < 1024 * 1024 { format!("{:.1} KB", bytes as f64 / 1024.0) }
    else if bytes < 1024 * 1024 * 1024 { format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0)) }
    else { format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0)) }
}
