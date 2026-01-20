use ratatui::{
    layout::{Rect, Alignment},
    style::{Color, Style, Modifier},
    widgets::{
        block::Title, Block, Borders, BorderType, Paragraph, Wrap, Chart, Dataset, Axis, GraphType
    },
    symbols,
    text::{Span, Line},
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

fn get_usage_color(usage: f64) -> Color {
    match usage {
        u if u < 60.0 => Color::Green,
        u if u < 80.0 => Color::Yellow,
        u if u < 95.0 => Color::LightRed,
        _ => Color::Red,
    }
}

fn calculate_trend(history: &[u64]) -> &'static str {
    if history.len() < 2 {
        return "→";
    }

    let recent = &history[history.len() - 2..];
    let current = recent[1] as f64;
    let previous = recent[0] as f64;

    if current > previous * 1.05 {
        "↗"
    } else if current < previous * 0.95 {
        "↘"
    } else {
        "→"
    }
}

fn get_peak_value(history: &[u64]) -> u64 {
    history.iter().cloned().max().unwrap_or(0)
}

fn get_peak_percent(history: &[u64], limit: u64) -> f64 {
    if limit == 0 {
        0.0
    } else {
        history.iter().map(|&v| (v as f64 / limit as f64) * 100.0).fold(0.0, |a, b| a.max(b))
    }
}

fn render_enhanced_graph(
    f: &mut Frame,
    area: Rect,
    name: Line,
    current_val_str: String,
    current_val_color: Color,
    is_critical: bool,
    datasets: Vec<Dataset>,
    y_max: f64,
    y_labels: Vec<String>,
) {
    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(current_val_color));

    // Title: Name (Left)
    block = block.title(
        Title::from(name)
            .alignment(Alignment::Left)
    );

    // Title: Value (Right)
    let val_style = if is_critical {
        Style::default().fg(current_val_color).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(current_val_color)
    };
    
    block = block.title(
        Title::from(Span::styled(current_val_str, val_style))
            .alignment(Alignment::Right)
    );

    let chart = Chart::new(datasets)
        .block(block)
        .x_axis(
            Axis::default()
                .bounds([0.0, 60.0])
                .labels(vec![Span::raw("60s")])
                .style(Style::default().fg(Color::DarkGray))
        )
        .y_axis(
            Axis::default()
                .bounds([0.0, y_max])
                .labels(y_labels.iter().map(|s| Span::raw(s)).collect::<Vec<_>>())
                .style(Style::default().fg(Color::DarkGray))
        );
        
    f.render_widget(chart, area);
}

pub fn render_container_details(f: &mut Frame<'_>, area: Rect, app: &App) {
    let details_lock = app.selected_container_details.read().unwrap();
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
    if let Some(container) = app.selected_container() {
        let stats_map = app.container_stats.read().unwrap();
        if let Some(stats) = stats_map.get(&container.id) {
            // Split graphs area: Left CPU, Right Memory
            let (cpu_area, mem_area) = get_graphs_layout(graphs_area);
            
            // --- CPU Graph ---
            let cpu_color = get_usage_color(stats.cpu_percent);
            let is_cpu_critical = stats.cpu_percent >= 95.0;
            let cpu_trend = calculate_trend(&stats.cpu_history);
            let cpu_peak = get_peak_value(&stats.cpu_history) as f64 / 100.0;
            
            // Title construction
            let cpu_title = Line::from(vec![
                Span::raw("CPU "),
                Span::styled(format!("[Peak: {:.1}%]", cpu_peak), Style::default().fg(Color::DarkGray))
            ]);
            
            let cpu_val_str = format!("{:.1}% {}", stats.cpu_percent, cpu_trend);

            // Data Preparation
            let cpu_data: Vec<(f64, f64)> = stats.cpu_history
                .iter()
                .enumerate()
                .map(|(i, &v)| (i as f64, v as f64 / 100.0))
                .collect();

            let user_cpu_data: Vec<(f64, f64)> = stats.user_cpu_history
                .iter()
                .enumerate()
                .map(|(i, &v)| (i as f64, v as f64 / 100.0))
                .collect();

            let system_cpu_data: Vec<(f64, f64)> = stats.system_cpu_history
                .iter()
                .enumerate()
                .map(|(i, &v)| (i as f64, v as f64 / 100.0))
                .collect();
            
            // Grid lines
            let grid_25 = vec![(0.0, 25.0), (60.0, 25.0)];
            let grid_50 = vec![(0.0, 50.0), (60.0, 50.0)];
            let grid_75 = vec![(0.0, 75.0), (60.0, 75.0)];

            let cpu_datasets = vec![
                // Grid Lines
                Dataset::default()
                    .marker(symbols::Marker::Braille)
                    .style(Style::default().fg(Color::DarkGray))
                    .graph_type(GraphType::Line)
                    .data(&grid_25),
                Dataset::default()
                    .marker(symbols::Marker::Braille)
                    .style(Style::default().fg(Color::DarkGray))
                    .graph_type(GraphType::Line)
                    .data(&grid_50),
                Dataset::default()
                    .marker(symbols::Marker::Braille)
                    .style(Style::default().fg(Color::DarkGray))
                    .graph_type(GraphType::Line)
                    .data(&grid_75),
                // Data Lines
                Dataset::default()
                    .name("System")
                    .marker(symbols::Marker::Braille)
                    .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::DIM))
                    .graph_type(GraphType::Line)
                    .data(&system_cpu_data),
                Dataset::default()
                    .name("User")
                    .marker(symbols::Marker::Braille)
                    .style(Style::default().fg(Color::Blue).add_modifier(Modifier::DIM))
                    .graph_type(GraphType::Line)
                    .data(&user_cpu_data),
                Dataset::default()
                    .name("Total")
                    .marker(symbols::Marker::Braille)
                    .style(Style::default().fg(cpu_color).add_modifier(Modifier::BOLD))
                    .graph_type(GraphType::Line)
                    .data(&cpu_data),
            ];

            // --- MEM Graph ---
            let mem_percent = if stats.memory_limit > 0 {
                (stats.memory_usage as f64 / stats.memory_limit as f64) * 100.0
            } else {
                0.0
            };
            
            let mem_color = get_usage_color(mem_percent);
            let is_mem_critical = mem_percent >= 95.0;
            let mem_trend = calculate_trend(&stats.memory_history);
            let mem_peak_percent = get_peak_percent(&stats.memory_history, stats.memory_limit);
            
            let mem_title = Line::from(vec![
                Span::raw("MEM "),
                Span::styled(format!("[Peak: {:.1}%]", mem_peak_percent), Style::default().fg(Color::DarkGray))
            ]);
            
            let mem_val_str = format!("{} {}", format_bytes(stats.memory_usage), mem_trend);

            let mem_data: Vec<(f64, f64)> = stats.memory_history
                .iter()
                .enumerate()
                .map(|(i, &v)| (i as f64, if stats.memory_limit > 0 { (v as f64 / stats.memory_limit as f64) * 100.0 } else { 0.0 }))
                .collect();

            let cached_mem_data: Vec<(f64, f64)> = stats.cached_memory_history
                .iter()
                .enumerate()
                .map(|(i, &v)| (i as f64, if stats.memory_limit > 0 { (v as f64 / stats.memory_limit as f64) * 100.0 } else { 0.0 }))
                .collect();

            let mem_datasets = vec![
                 // Grid Lines
                Dataset::default()
                    .marker(symbols::Marker::Braille)
                    .style(Style::default().fg(Color::DarkGray))
                    .graph_type(GraphType::Line)
                    .data(&grid_25),
                Dataset::default()
                    .marker(symbols::Marker::Braille)
                    .style(Style::default().fg(Color::DarkGray))
                    .graph_type(GraphType::Line)
                    .data(&grid_50),
                Dataset::default()
                    .marker(symbols::Marker::Braille)
                    .style(Style::default().fg(Color::DarkGray))
                    .graph_type(GraphType::Line)
                    .data(&grid_75),
                Dataset::default()
                    .name("Cached")
                    .marker(symbols::Marker::Braille)
                    .style(Style::default().fg(Color::Blue).add_modifier(Modifier::DIM))
                    .graph_type(GraphType::Line)
                    .data(&cached_mem_data),
                Dataset::default()
                    .name("Used")
                    .marker(symbols::Marker::Braille)
                    .style(Style::default().fg(mem_color).add_modifier(Modifier::BOLD))
                    .graph_type(GraphType::Line)
                    .data(&mem_data),
            ];

            // Render
            render_enhanced_graph(f, cpu_area, cpu_title, cpu_val_str, cpu_color, is_cpu_critical, cpu_datasets, 100.0, vec!["0".into(), "50".into(), "100".into()]);
            render_enhanced_graph(f, mem_area, mem_title, mem_val_str, mem_color, is_mem_critical, mem_datasets, 100.0, vec!["0".into(), "50".into(), "100".into()]);
        }
    }
}
