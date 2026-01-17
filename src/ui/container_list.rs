use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style, Stylize},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
    layout::Constraint,
};
use chrono::Utc;
use crate::app::App;

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
    let containers = app.containers.read().unwrap();
    
    // Header cells - simplified for compact view if needed, but we have space
    let header_cells = ["NAME", "STATUS", "IMG", "UP", "CPU / MEM"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Black).bg(Color::Cyan).bold()));
    let header = Row::new(header_cells).height(1);
    
    let stats_map = app.container_stats.read().unwrap();

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
            Cell::from(image),
            Cell::from(uptime),
            Cell::from(stats_str),
        ];
        Row::new(cells).height(1)
    });

    // Adjust constraints for the list columns
    let widths = [
        Constraint::Percentage(25),
        Constraint::Percentage(15),
        Constraint::Percentage(20),
        Constraint::Percentage(10),
        Constraint::Percentage(30),
    ];

    let border_style = if app.focus == crate::app::Focus::ContainerList {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Magenta)
    };

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Containers ({}) ", app.total_containers))
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
