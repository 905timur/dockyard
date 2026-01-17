use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style, Stylize},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
    layout::Constraint,
};
use chrono::{DateTime, Utc};
use crate::app::App;

fn format_bytes(bytes: u64) -> String {
    const GB: u64 = 1024 * 1024 * 1024;
    const MB: u64 = 1024 * 1024;

    if bytes >= GB {
        format!("{:.2}GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1}MB", bytes as f64 / MB as f64)
    } else {
        format!("{:.1}KB", bytes as f64 / 1024.0)
    }
}

fn format_time(timestamp: i64) -> String {
    // timestamp is unix timestamp
    let dt = DateTime::<Utc>::from_timestamp(timestamp, 0).unwrap_or_default();
    let now = Utc::now();
    let duration = now.signed_duration_since(dt);
    
    if duration.num_days() > 0 {
        format!("{}d ago", duration.num_days())
    } else if duration.num_hours() > 0 {
        format!("{}h ago", duration.num_hours())
    } else {
        format!("{}m ago", duration.num_minutes())
    }
}

pub fn render_image_list(f: &mut Frame<'_>, area: Rect, app: &mut App) {
    let images = app.images.read().unwrap();
    
    let header_cells = ["REPOSITORY", "TAG", "IMAGE ID", "SIZE", "CREATED"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Black).bg(Color::Cyan).bold()));
    let header = Row::new(header_cells).height(1);

    let rows = images.iter().map(|i| {
        let (repo, tag) = if let Some(first_tag) = i.repo_tags.first() {
            // Check if tag is literally "<none>:<none>" which bollard might return
            if first_tag == "<none>:<none>" {
                 ("<none>".to_string(), "<none>".to_string())
            } else if let Some((r, t)) = first_tag.split_once(':') {
                (r.to_string(), t.to_string())
            } else {
                (first_tag.clone(), "<none>".to_string())
            }
        } else {
            ("<none>".to_string(), "<none>".to_string())
        };

        let cells = vec![
            Cell::from(repo).style(Style::default().fg(Color::Cyan)),
            Cell::from(tag),
            Cell::from(i.id.clone()),
            Cell::from(format_bytes(i.size as u64)),
            Cell::from(format_time(i.created)),
        ];
        Row::new(cells).height(1)
    });

    let widths = [
        Constraint::Percentage(30),
        Constraint::Percentage(20),
        Constraint::Percentage(15),
        Constraint::Percentage(15),
        Constraint::Percentage(20),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Images ({}) - Space: {} ", app.total_images, format_bytes(app.total_image_size)))
                .border_style(Style::default().fg(Color::Magenta))
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    f.render_stateful_widget(table, area, &mut app.table_state_images);
}
