use bollard::models::{ContainerInspectResponse, HealthStatusEnum as BollardHealthStatus};
use crate::types::{ContainerHealth, HealthStatus, HealthCheckResult, Result};
use crate::docker::client::DockerClient;
use crate::docker::containers::inspect_container;
use chrono::{DateTime, Utc};
use std::collections::VecDeque;

pub async fn fetch_health_info(client: &DockerClient, id: &str) -> Result<ContainerHealth> {
    let inspect = inspect_container(client, id).await?;
    parse_health_info(inspect)
}

pub fn parse_health_status_from_string(status: &str) -> HealthStatus {
    let status = status.to_lowercase();
    if status.contains("(healthy)") {
        HealthStatus::Healthy
    } else if status.contains("(unhealthy)") {
        HealthStatus::Unhealthy
    } else if status.contains("(health: starting)") {
        HealthStatus::Starting
    } else {
        HealthStatus::NoHealthCheck // Or Unknown, but usually if not present it means no check
    }
}

fn parse_health_info(inspect: ContainerInspectResponse) -> Result<ContainerHealth> {
    let state = inspect.state.as_ref();
    let health = state.and_then(|s| s.health.as_ref());
    let config = inspect.config.as_ref().and_then(|c| c.healthcheck.as_ref());

    if let Some(health_data) = health {
        let status = match health_data.status {
            Some(BollardHealthStatus::NONE) => HealthStatus::NoHealthCheck,
            Some(BollardHealthStatus::STARTING) => HealthStatus::Starting,
            Some(BollardHealthStatus::HEALTHY) => HealthStatus::Healthy,
            Some(BollardHealthStatus::UNHEALTHY) => HealthStatus::Unhealthy,
            _ => HealthStatus::NoHealthCheck,
        };

        let failing_streak = health_data.failing_streak.unwrap_or(0) as u64;
        
        let mut check_history = VecDeque::new();
        if let Some(log) = &health_data.log {
            for entry in log.iter().rev().take(5) {
                if let (Some(start), Some(exit), Some(out)) = (&entry.start, entry.exit_code, &entry.output) {
                    // entry.start is String in ISO format usually, bollard might have it as String
                    // We need to parse it. Bollard models define it as String.
                    if let Ok(ts) = DateTime::parse_from_rfc3339(start) {
                        check_history.push_front(HealthCheckResult {
                            timestamp: ts.with_timezone(&Utc),
                            exit_code: exit.clone(),
                            output: out.chars().take(200).collect(),
                        });
                    }
                }
            }
        }

        let last_check = check_history.back();
        let last_check_at = last_check.map(|c| c.timestamp);
        let last_check_output = last_check.map(|c| c.output.clone());

        // Config info
        let retries = config.and_then(|c| c.retries);

        // Helper to format duration string nicely (nano to readable)
        fn format_duration(ns: i64) -> String {
            if ns == 0 { return "0s".to_string(); }
            let secs = ns / 1_000_000_000;
            if secs >= 60 {
                if secs % 60 == 0 {
                    format!("{}m", secs / 60)
                } else {
                    format!("{}m {}s", secs / 60, secs % 60)
                }
            } else {
                format!("{}s", secs)
            }
        }

        let interval = config.and_then(|c| c.interval).map(format_duration);
        let timeout = config.and_then(|c| c.timeout).map(format_duration);
        let start_period = config.and_then(|c| c.start_period).map(format_duration);

        Ok(ContainerHealth {
            status,
            failing_streak,
            last_check_at,
            last_check_output,
            check_history,
            interval,
            timeout,
            retries,
            start_period,
        })
    } else {
        // No health data found
        Ok(ContainerHealth {
            status: HealthStatus::NoHealthCheck,
            failing_streak: 0,
            last_check_at: None,
            last_check_output: None,
            check_history: VecDeque::new(),
            interval: None,
            timeout: None,
            retries: None,
            start_period: None,
        })
    }
}
