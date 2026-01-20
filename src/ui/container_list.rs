use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style, Stylize},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
    layout::Constraint,
};
use chrono::Utc;
use crate::app::App;
use crate::types::HealthStatus;

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

pub fn render_container_list(f: &mut Frame<'_>, area: Rect, app: &mut App) {
    // Ensure filtered list is up to date with any background changes
    app.update_filtered_containers();
    
    // We clone here to avoid borrow issues since we need immutable borrow for summary and rows
    // but filtered_containers is a field on app.
    // Actually, we can just access app.filtered_containers.
    // But summary calculation needs app.containers.
    
    let containers_lock = app.containers.read().unwrap();
    
    // Header cells - simplified for compact view if needed, but we have space
    let header_cells = ["NAME", "STATUS", "HEALTH", "IMG", "UP", "CPU / MEM"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Black).bg(Color::Cyan).bold()));
    let header = Row::new(header_cells).height(1);
    
    let stats_map = app.container_stats.read().unwrap();
    let health_map = app.container_health.read().unwrap();

    // Calculate Summary (based on ALL running containers, not filtered)
    let mut healthy_count = 0;
    let mut unhealthy_count = 0;
    let mut starting_count = 0;

    for c in containers_lock.iter() {
        if c.state == "running" {
            if let Some(h) = health_map.get(&c.id) {
                match h.status {
                    HealthStatus::Healthy => healthy_count += 1,
                    HealthStatus::Unhealthy => unhealthy_count += 1,
                    HealthStatus::Starting => starting_count += 1,
                    _ => {}
                }
            }
        }
    }

    // Use filtered containers for display
    let rows = app.filtered_containers.iter().map(|c| {
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

        // Health
        let health_cell = if c.state == "running" {
            if let Some(h) = health_map.get(&c.id) {
                match h.status {
                    HealthStatus::Healthy => Cell::from("✓ healthy").style(Style::default().fg(Color::Green)),
                    HealthStatus::Unhealthy => {
                        let text = if h.failing_streak > 0 {
                            format!("✗ failing({})", h.failing_streak)
                        } else {
                            "✗ unhealthy".to_string()
                        };
                        Cell::from(text).style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
                    },
                    HealthStatus::Starting => Cell::from("⚠ starting").style(Style::default().fg(Color::Yellow)),
                    HealthStatus::NoHealthCheck => Cell::from("-").style(Style::default().fg(Color::DarkGray)),
                    HealthStatus::Unknown => Cell::from("?").style(Style::default().fg(Color::Magenta)),
                }
            } else {
                Cell::from("...")
            }
        } else {
            Cell::from("-")
        };

        // Shorten image name
        let image = if c.image.len() > 15 {
             format!("{}...", &c.image[0..12])
        } else {
             c.image.clone()
        };
        
        // Stats
        let stats_str = if c.state == "running" {
            if let Some(stats) = stats_map.get(&c.id) {
                let is_stale = Utc::now().timestamp() - stats.last_updated > 10;
                let mem_str = format_bytes(stats.memory_usage);
                if is_stale {
                     format!("(stale) {:.1}% / {}", stats.cpu_percent, mem_str)
                } else {
                     format!("{:.1}% / {}", stats.cpu_percent, mem_str)
                }
            } else {
                "Fetching...".to_string()
            }
        } else {
            "-".to_string()
        };

        let cells = vec![
            Cell::from(c.name.clone()).style(Style::default().fg(Color::Cyan)),
            Cell::from(format!("{} {}", status_symbol, c.state))
                .style(Style::default().fg(status_color).bold()),
            health_cell,
            Cell::from(image),
            Cell::from(uptime),
            Cell::from(stats_str),
        ];
        Row::new(cells).height(1)
    });

    // Adjust constraints for the list columns
    let widths = [
        Constraint::Percentage(20),
        Constraint::Percentage(10),
        Constraint::Percentage(15),
        Constraint::Percentage(20),
        Constraint::Percentage(10),
        Constraint::Percentage(25),
    ];

    let border_style = if app.focus == crate::app::Focus::ContainerList {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Magenta)
    };

    let title = if unhealthy_count > 0 || starting_count > 0 || healthy_count > 0 {
        format!(" Containers ({}) | Health: ✓{} ⚠{} ✗{} ", app.total_containers, healthy_count, starting_count, unhealthy_count)
    } else {
        format!(" Containers ({}) ", app.total_containers)
    };

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(border_style)
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    f.render_stateful_widget(table, area, &mut app.table_state);

    // Update viewport state for background fetching
    let height = area.height.saturating_sub(2); // Subtract borders
    let offset = app.table_state.offset();
    
    if let Ok(mut viewport) = app.viewport_state.write() {
        viewport.height = height;
        viewport.offset = offset;
    }
}
