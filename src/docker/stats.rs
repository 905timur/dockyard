use crate::docker::client::DockerClient;
use crate::types::Result;
use bollard::container::StatsOptions;
use futures::StreamExt;

pub async fn fetch_container_stats(
    client: &DockerClient,
    id: &str,
) -> Result<Option<(f64, f64, f64, u64, u64, u64)>> {
    let mut stats_stream = client.inner.stats(
        id,
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

        // Calculate user and system CPU percentages
        let user_cpu_delta = stats.cpu_stats.cpu_usage.usage_in_usermode
            .saturating_sub(stats.precpu_stats.cpu_usage.usage_in_usermode);
        let system_cpu_delta = stats.cpu_stats.cpu_usage.usage_in_kernelmode
            .saturating_sub(stats.precpu_stats.cpu_usage.usage_in_kernelmode);

        let user_cpu_percent = if system_delta > 0 && user_cpu_delta > 0 {
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
            (user_cpu_delta as f64 / system_delta as f64) * num_cpus as f64 * 100.0
        } else {
            0.0
        };

        let system_cpu_percent = if system_delta > 0 && system_cpu_delta > 0 {
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
            (system_cpu_delta as f64 / system_delta as f64) * num_cpus as f64 * 100.0
        } else {
            0.0
        };

        let memory_usage = stats.memory_stats.usage.unwrap_or(0);
        let cached_memory = if let Some(stats_map) = &stats.memory_stats.stats {
             match stats_map {
                 bollard::container::MemoryStatsStats::V1(v1) => v1.cache,
                 bollard::container::MemoryStatsStats::V2(_v2) => 0, // V2 doesn't have cache field
             }
        } else {
             0
        };
        let memory_limit = stats.memory_stats.limit.unwrap_or(0);

        Ok(Some((cpu_percent, user_cpu_percent, system_cpu_percent, memory_usage, cached_memory, memory_limit)))
    } else {
        Ok(None)
    }
}
