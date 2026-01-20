use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::VecDeque;

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
