use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Sparkline, Wrap},
    Frame,
};
use crate::app::App;
use crate::ui::layout::{get_details_layout, get_graphs_layout};

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

pub async fn render_container_details(f: &mut Frame<'_>, area: Rect, app: &App) {
    let details_lock = app.selected_container_details.read().await;
    let details_text = match details_lock.as_ref() {
        Some(text) => text.clone(),
        None => "Select a container to view details".to_string(),
    };
    drop(details_lock);

    // Split area: Top for text, Bottom for graphs
    let (text_area, graphs_area) = get_details_layout(area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Details ")
        .border_style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(details_text)
        .block(block)
        .wrap(Wrap { trim: true });
    
    f.render_widget(paragraph, text_area);

    // Render Graphs if a container is selected
    if let Some(container) = app.selected_container().await {
        let stats_map = app.container_stats.read().await;
        if let Some(stats) = stats_map.get(&container.id) {
            // Split graphs area: Left CPU, Right Memory
            let (cpu_area, mem_area) = get_graphs_layout(graphs_area);
            
            // CPU Graph
            let cpu_block = Block::default()
                .borders(Borders::ALL)
                .title(format!(" CPU: {:.2}% ", stats.cpu_percent));
            
            let cpu_sparkline = Sparkline::default()
                .block(cpu_block)
                .data(&stats.cpu_history)
                .style(Style::default().fg(Color::Green));
            
            f.render_widget(cpu_sparkline, cpu_area);

            // Memory Graph
            let mem_block = Block::default()
                .borders(Borders::ALL)
                .title(format!(" MEM: {} ", format_bytes(stats.memory_usage)));
            
            let mem_sparkline = Sparkline::default()
                .block(mem_block)
                .data(&stats.memory_history)
                .style(Style::default().fg(Color::Magenta));
             
            f.render_widget(mem_sparkline, mem_area);
        }
    }
}
