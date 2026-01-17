use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem},
    text::{Line, Span},
    Frame,
};
use crate::app::App;

pub fn render_container_logs(f: &mut Frame<'_>, area: Rect, app: &App) {
    let logs_lock = app.selected_container_logs.read().unwrap();
    
    let logs_items: Vec<ListItem> = logs_lock
        .iter()
        .map(|log| {
             let lower = log.to_lowercase();
             let style = if lower.contains("error") {
                 Style::default().fg(Color::Red)
             } else if lower.contains("warn") {
                 Style::default().fg(Color::Yellow)
             } else if lower.contains("info") {
                 Style::default().fg(Color::Green)
             } else {
                 Style::default().fg(Color::White)
             };
             ListItem::new(Line::from(Span::styled(log.as_str(), style)))
        })
        .collect();

    let title = if app.auto_scroll {
        " Logs (Live - Auto Scroll) "
    } else {
        " Logs (Live - Manual Scroll) "
    };

    let border_style = if app.focus == crate::app::Focus::Logs {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Yellow)
    };

    let logs_list = List::new(logs_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(border_style)
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    let mut state = app.logs_state.clone();
    f.render_stateful_widget(logs_list, area, &mut state);
}
