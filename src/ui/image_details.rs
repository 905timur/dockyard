use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use crate::app::App;

pub fn render_image_details(f: &mut Frame<'_>, area: Rect, app: &App) {
    let details_lock = app.selected_image_details.read().unwrap();
    let details_text = match details_lock.as_ref() {
        Some(text) => text.clone(),
        None => "Select an image to view details".to_string(),
    };
    drop(details_lock);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Image Inspection ")
        .border_style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(details_text)
        .block(block)
        .wrap(Wrap { trim: true });
    
    f.render_widget(paragraph, area);
}

pub fn render_image_context(f: &mut Frame<'_>, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Output ")
        .border_style(Style::default().fg(Color::Cyan));

    // Check if pulling
    if !app.pull_progress.read().unwrap().is_empty() || app.is_pulling.load(std::sync::atomic::Ordering::Relaxed) {
         let progress = app.pull_progress.read().unwrap();
         // Show last few lines
         let progress_text: String = progress.iter().rev().take(10).rev().cloned().collect::<Vec<String>>().join("\n");
         
         let paragraph = Paragraph::new(progress_text)
            .block(block.title(" Pull Progress "))
            .wrap(Wrap { trim: true });
         f.render_widget(paragraph, area);
    } else {
        // Idle or show errors?
        let paragraph = Paragraph::new("Idle.")
            .block(block)
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(paragraph, area);
    }
}

pub fn render_pull_dialog(f: &mut Frame<'_>, area: Rect, app: &App) {
    if !app.show_pull_dialog {
        return;
    }

    let area = centered_rect(50, 10, area); // Smaller height for just input
    f.render_widget(Clear, area);
    
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .title(" Pull Image (Esc to cancel, Enter to pull) ");
        
    f.render_widget(block, area);

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Min(1)])
        .split(area)[0];

    // Input
    let input_text = format!("> {}", app.pull_input);
    let input = Paragraph::new(input_text);
    f.render_widget(input, inner);
}

pub fn render_delete_confirm(f: &mut Frame<'_>, area: Rect, app: &App) {
    if !app.show_delete_confirm {
        return;
    }
    
    let area = centered_rect(40, 10, area);
    f.render_widget(Clear, area);
    
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red))
        .title(" Confirm Deletion ");
        
    let text = "Are you sure you want to delete the selected image?\nPress 'y' to confirm, 'n' or Esc to cancel.";
    let p = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
    
    f.render_widget(p, area);
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
