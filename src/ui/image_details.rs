use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use crate::app::App;

pub fn render_image_details(f: &mut Frame<'_>, area: Rect, app: &App) {
    let details = app.selected_image_details.read().unwrap();
    if let Some(text) = details.as_ref() {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Image Details (Esc to close) ")
            .border_style(Style::default().fg(Color::Cyan));
        
        let area = centered_rect(70, 70, area);
        
        f.render_widget(Clear, area);
        
        let paragraph = Paragraph::new(text.as_str())
            .block(block)
            .wrap(Wrap { trim: true })
            .scroll((0, 0)); // We might need scroll state later if content is long
            
        f.render_widget(paragraph, area);
    }
}

pub fn render_pull_dialog(f: &mut Frame<'_>, area: Rect, app: &App) {
    if !app.show_pull_dialog && !app.is_pulling.load(std::sync::atomic::Ordering::Relaxed) {
        return;
    }

    let area = centered_rect(50, 40, area);
    f.render_widget(Clear, area);
    
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .title(" Pull Image ");
        
    f.render_widget(block.clone(), area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Input
            Constraint::Min(1),    // Progress
        ])
        .split(area);

    // Input
    let input_text = format!("Image: {}", app.pull_input);
    let input = Paragraph::new(input_text)
        .block(Block::default().borders(Borders::ALL).title(" Repository[:Tag] "));
    f.render_widget(input, chunks[0]);
    
    // Progress
    let progress = app.pull_progress.read().unwrap();
    let progress_text: String = progress.join("\n");
    let progress_widget = Paragraph::new(progress_text)
        .block(Block::default().borders(Borders::ALL).title(" Progress "));
    f.render_widget(progress_widget, chunks[1]);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
