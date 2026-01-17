use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    text::Line,
    Frame,
};

pub fn render_help(f: &mut Frame<'_>, area: Rect) {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(area);

    let popup_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(popup_layout[1])[1];

    let help_text = vec![
        Line::from("        dockyard        "),
        Line::from("         v0.2.0         "),
        Line::from(""),
        Line::from("Navigation:"),
        Line::from("  ↑/↓ or j/k: Navigate (List/Logs)"),
        Line::from("  Tab: Toggle Focus (List/Logs)"),
        Line::from("  Shift+Tab / v: Switch View (Containers/Images)"),
        Line::from("  Esc or q: Close help / Quit"),
        Line::from(""),
        Line::from("Actions:"),
        Line::from("  s: Stop container"),
        Line::from("  t: Start container"),
        Line::from("  r: Restart container"),
        Line::from("  p: Pause container"),
        Line::from("  u: Unpause container"),
        Line::from("  e: Exec shell"),
        Line::from("  d: Remove container"),
        Line::from("  f: Toggle filter (All/Running)"),
        Line::from("  a: Toggle auto-scroll logs"),
        Line::from("  J/K: Scroll logs manually"),
        Line::from(""),
        Line::from("General:"),
        Line::from("  ?: Toggle this help menu"),
    ];

    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(help_text)
        .block(block)
        .wrap(Wrap { trim: true });

    f.render_widget(Clear, popup_area);
    f.render_widget(paragraph, popup_area);
}
