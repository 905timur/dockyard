use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Modifier},
    widgets::{Block, Borders, Clear, Row, Table, Cell, Paragraph},
    Frame,
};

pub fn render_help(f: &mut Frame<'_>, area: Rect) {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Percentage(80),
            Constraint::Percentage(10),
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
        
    // Split popup area into header and content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Header
            Constraint::Min(0),    // Table
        ])
        .split(popup_area);

    f.render_widget(Clear, popup_area);
    
    // Render Header
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
        
    let header_text = vec![
        ratatui::text::Line::from("Dockyard"),
        ratatui::text::Line::from("v0.2.1"),
    ];
    
    let header = Paragraph::new(header_text)
        .block(block)
        .alignment(Alignment::Center)
        .style(Style::default().add_modifier(Modifier::BOLD));
        
    f.render_widget(header, chunks[0]);

    // Render Table
    let rows = vec![
        // Performance Modes
        Row::new(vec![
            Cell::from("PERFORMANCE").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from(""),
        ]),
        Row::new(vec!["t", "Toggle Turbo/Normal mode"]),
        Row::new(vec!["m", "Toggle stats view (detailed/minimal)"]),
        Row::new(vec!["[ ]", "Adjust refresh interval"]),
        Row::new(vec!["P", "Show performance metrics"]),
        Row::new(vec!["1/2/3", "Quick presets (1=Max, 2=Balanced, 3=Detail)"]),
        Row::new(vec!["", ""]), // Spacer

        // Navigation
        Row::new(vec![
            Cell::from("NAVIGATION").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from(""),
        ]),
        Row::new(vec!["↑/↓ or j/k", "Navigate List/Logs"]),
        Row::new(vec!["Tab", "Toggle Focus (List/Logs)"]),
        Row::new(vec!["Shift+Tab / v", "Switch View (Containers/Images)"]),
        Row::new(vec!["Esc / q", "Close Help / Quit"]),
        Row::new(vec!["?", "Toggle Help"]),
        Row::new(vec!["R", "Manual refresh"]),
        Row::new(vec!["", ""]), // Spacer

        // Container Actions
        Row::new(vec![
            Cell::from("CONTAINER ACTIONS").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from(""),
        ]),
        Row::new(vec!["s", "Stop Container"]),
        Row::new(vec!["t", "Start Container"]),
        Row::new(vec!["r", "Restart Container"]),
        Row::new(vec!["p", "Pause Container"]),
        Row::new(vec!["u", "Unpause Container"]),
        Row::new(vec!["e", "Exec Shell"]),
        Row::new(vec!["d", "Remove Container"]),
        Row::new(vec!["f", "Toggle Filter (All/Running)"]),
        
        // Logs
        Row::new(vec![
            Cell::from("LOGS").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from(""),
        ]),
        Row::new(vec!["a", "Toggle Auto-scroll"]),
        Row::new(vec!["J/K", "Scroll Logs Manually"]),

        // Image Actions
         Row::new(vec![
            Cell::from("IMAGE ACTIONS").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from(""),
        ]),
        Row::new(vec!["p", "Pull Image"]),
        Row::new(vec!["d", "Remove Image"]),
        Row::new(vec!["D", "Force Remove Image"]),
        Row::new(vec!["Enter", "View Image Details"]),
        Row::new(vec!["f", "Toggle Dangling Images"]),
        Row::new(vec!["s", "Sort Images"]),
    ];

    let table = Table::new(rows, [
        Constraint::Percentage(40),
        Constraint::Percentage(60),
    ])
    .block(Block::default()
        .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
        .border_style(Style::default().fg(Color::Cyan)))
    .column_spacing(1);

    f.render_widget(table, chunks[1]);
}
