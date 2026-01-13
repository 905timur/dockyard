use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub fn get_main_layout(area: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(75),
        ])
        .split(area);
    (chunks[0], chunks[1])
}

pub fn get_right_pane_layout(area: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(area);
    (chunks[0], chunks[1])
}

pub fn get_details_layout(area: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10), // Text area
            Constraint::Length(10), // Graphs area
        ])
        .split(area);
    (chunks[0], chunks[1])
}

pub fn get_graphs_layout(area: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(area);
    (chunks[0], chunks[1])
}
