use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::VecDeque;
use std::time::Duration;

// --- Configuration Types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub turbo_mode: bool,
    pub refresh_rate: RefreshRate,
    pub stats_view: StatsView,
    pub poll_strategy: PollStrategy,
    pub viewport_buffer: usize,
    pub show_perf_metrics: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            turbo_mode: false,
            refresh_rate: RefreshRate::Interval(Duration::from_secs(1)),
            stats_view: StatsView::Detailed,
            poll_strategy: PollStrategy::AllContainers,
            viewport_buffer: 5,
            show_perf_metrics: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "duration")]
pub enum RefreshRate {
    Interval(Duration),
    Manual,
}

impl RefreshRate {
    pub fn display(&self) -> String {
        match self {
            RefreshRate::Interval(d) => format!("{}s", d.as_secs()),
            RefreshRate::Manual => "Manual".to_string(),
        }
    }

    pub fn increase(&mut self, _is_turbo: bool) {
        match self {
            RefreshRate::Interval(d) => {
                let secs = d.as_secs();
                let next = match secs {
                    1 => 2,
                    2 => 5,
                    5 => 10,
                    10 => 30,
                    30 => {
                        *self = RefreshRate::Manual;
                        return;
                    }
                    _ => 2,
                };
                *self = RefreshRate::Interval(Duration::from_secs(next));
            }
            RefreshRate::Manual => {} // Maxed out
        }
    }

    pub fn decrease(&mut self, is_turbo: bool) {
        match self {
            RefreshRate::Manual => {
                *self = RefreshRate::Interval(Duration::from_secs(30));
            }
            RefreshRate::Interval(d) => {
                let secs = d.as_secs();
                let min = if is_turbo { 2 } else { 1 };
                let next = match secs {
                    30 => 10,
                    10 => 5,
                    5 => 2,
                    2 => 1,
                    _ => 1,
                };
                if next >= min {
                    *self = RefreshRate::Interval(Duration::from_secs(next));
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StatsView {
    Detailed,
    Minimal,
}

impl StatsView {
    pub fn toggle(&mut self) {
        *self = match self {
            StatsView::Detailed => StatsView::Minimal,
            StatsView::Minimal => StatsView::Detailed,
        };
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "buffer")]
pub enum PollStrategy {
    AllContainers,
    VisibleOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerInfo {
    pub id: String,
    pub short_id: String,
    pub name: String,
    pub status: String,
    pub image: String,
    pub ports: String,
    pub created: i64,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageInfo {
    pub id: String,
    pub repo_tags: Vec<String>,
    pub size: i64,
    pub created: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerStats {
    pub cpu_percent: f64,
    pub user_cpu_percent: f64,
    pub system_cpu_percent: f64,
    pub memory_usage: u64,
    pub cached_memory: u64,
    pub memory_limit: u64,
    pub cpu_history: Vec<u64>,
    pub user_cpu_history: Vec<u64>,
    pub system_cpu_history: Vec<u64>,
    pub memory_history: Vec<u64>,
    pub cached_memory_history: Vec<u64>,
    pub last_updated: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum HealthStatus {
    Unhealthy,        // Current check failed
    Starting,         // Container initializing, checks not yet run
    Healthy,          // All checks passing
    NoHealthCheck,    // No health check configured
    Unknown,          // Unable to determine (API error, permission issue)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub timestamp: DateTime<Utc>,
    pub exit_code: i64,
    pub output: String,  // Truncated
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerHealth {
    pub status: HealthStatus,
    pub failing_streak: u64,
    pub last_check_at: Option<DateTime<Utc>>,
    pub last_check_output: Option<String>,  // Truncated to 200 chars
    pub check_history: VecDeque<HealthCheckResult>,  // Last 5 results
    // Config info
    pub interval: Option<String>,
    pub timeout: Option<String>,
    pub retries: Option<i64>,
    pub start_period: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpTab {
    Keybindings,
    Wiki,
}

impl Default for HelpTab {
    fn default() -> Self {
        Self::Keybindings
    }
}

#[derive(Debug, Default, Clone)]
pub struct PerfMetrics {
    pub cpu_usage: f64,
    pub memory_usage: u64,
    pub poll_time_ms: u64,
}

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Docker error: {0}")]
    Docker(#[from] bollard::errors::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Other error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, AppError>;
