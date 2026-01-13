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

pub async fn render_container_list(f: &mut Frame<'_>, area: Rect, app: &App) {
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
