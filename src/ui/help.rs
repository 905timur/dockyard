use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Modifier},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    text::{Line, Span},
    Frame,
};
use crate::app::App;
use crate::types::HelpTab;

pub fn render_help(f: &mut Frame<'_>, area: Rect, app: &App) {
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
            Constraint::Percentage(15),
            Constraint::Percentage(70),
            Constraint::Percentage(15),
        ])
        .split(popup_layout[1])[1];
        
    f.render_widget(Clear, popup_area);
    
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .title(" Help ");
    f.render_widget(block, popup_area);

    let inner_area = popup_area.inner(ratatui::layout::Margin { vertical: 1, horizontal: 1 });
    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title "Dockyard v0.3.0"
            Constraint::Length(3), // Tabs
            Constraint::Min(0),    // Content
            Constraint::Length(1), // Footer msg
        ])
        .split(inner_area);

    // Title
    let header_text = Line::from(vec![
        Span::styled("Dockyard", Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)),
        Span::raw(" v0.3.0"),
    ]);
    let header = Paragraph::new(header_text).alignment(Alignment::Center);
    f.render_widget(header, inner_chunks[0]);

    // Tabs
    let tab_titles = vec!["Keybindings", "Wiki"];
    let tabs = ratatui::widgets::Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Color::DarkGray)))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .select(match app.current_help_tab {
            HelpTab::Keybindings => 0,
            HelpTab::Wiki => 1,
        })
        .divider(" | ");
    f.render_widget(tabs, inner_chunks[1]);

    // Content
    match app.current_help_tab {
        HelpTab::Keybindings => render_keybindings(f, inner_chunks[2], app.help_scroll),
        HelpTab::Wiki => render_wiki(f, inner_chunks[2], app.help_scroll),
    }

    // Footer
    let footer_text = Line::from(vec![
        Span::styled("Tab", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(": Switch Tab | "),
        Span::styled("Up/Down", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(": Scroll | "),
        Span::styled("Esc/q", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(": Close"),
    ]);
    
    let footer = Paragraph::new(footer_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(footer, inner_chunks[3]);
}

fn render_keybindings(f: &mut Frame<'_>, area: Rect, scroll: u16) {
    let mut lines = Vec::new();

    // PERFORMANCE PRESETS
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled("PERFORMANCE PRESETS", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]));
    lines.push(Line::from(vec![Span::styled(format!("{: <12}", "1"), Style::default().fg(Color::Yellow)), Span::raw("Max Performance (Turbo + Manual Refresh + Minimal Stats)")]));
    lines.push(Line::from(vec![Span::styled(format!("{: <12}", "2"), Style::default().fg(Color::Yellow)), Span::raw("Balanced (Normal + 5s Interval + Minimal Stats)")]));
    lines.push(Line::from(vec![Span::styled(format!("{: <12}", "3"), Style::default().fg(Color::Yellow)), Span::raw("Full Detail (Normal + 1s Interval + Detailed Stats)")]));

    // PERFORMANCE CONTROLS
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled("PERFORMANCE CONTROLS", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]));
    lines.push(Line::from(vec![Span::styled(format!("{: <12}", "t"), Style::default().fg(Color::Yellow)), Span::raw("Toggle Turbo/Normal mode")]));
    lines.push(Line::from(vec![Span::styled(format!("{: <12}", "m"), Style::default().fg(Color::Yellow)), Span::raw("Toggle stats view (detailed/minimal)")]));
    lines.push(Line::from(vec![Span::styled(format!("{: <12}", "["), Style::default().fg(Color::Yellow)), Span::raw("Decrease refresh interval")]));
    lines.push(Line::from(vec![Span::styled(format!("{: <12}", "]"), Style::default().fg(Color::Yellow)), Span::raw("Increase refresh interval")]));
    lines.push(Line::from(vec![Span::styled(format!("{: <12}", "P"), Style::default().fg(Color::Yellow)), Span::raw("Show performance metrics (CPU/Memory)")]));

    // GLOBAL KEYS
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled("GLOBAL KEYS", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]));
    lines.push(Line::from(vec![Span::styled(format!("{: <12}", "?"), Style::default().fg(Color::Yellow)), Span::raw("Help menu")]));
    lines.push(Line::from(vec![Span::styled(format!("{: <12}", "Tab"), Style::default().fg(Color::Yellow)), Span::raw("Switch focus (Containers) or Switch Help Tab (Help Menu)")]));
    lines.push(Line::from(vec![Span::styled(format!("{: <12}", "Sh+Tab/v"), Style::default().fg(Color::Yellow)), Span::raw("Switch between Containers and Images views")]));
    lines.push(Line::from(vec![Span::styled(format!("{: <12}", "q"), Style::default().fg(Color::Yellow)), Span::raw("Quit")]));
    lines.push(Line::from(vec![Span::styled(format!("{: <12}", "R"), Style::default().fg(Color::Yellow)), Span::raw("Refresh containers and images manually")]));

    // CONTAINER VIEW
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled("CONTAINER VIEW", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]));
    lines.push(Line::from(vec![Span::styled(format!("{: <12}", "Up/Down"), Style::default().fg(Color::Yellow)), Span::raw("Navigate containers")]));
    lines.push(Line::from(vec![Span::styled(format!("{: <12}", "Enter"), Style::default().fg(Color::Yellow)), Span::raw("View detailed container info")]));
    lines.push(Line::from(vec![Span::styled(format!("{: <12}", "i"), Style::default().fg(Color::Yellow)), Span::raw("View resource history graphs")]));
    lines.push(Line::from(vec![Span::styled(format!("{: <12}", "l"), Style::default().fg(Color::Yellow)), Span::raw("View container logs")]));
    lines.push(Line::from(vec![Span::styled(format!("{: <12}", "e"), Style::default().fg(Color::Yellow)), Span::raw("Launch interactive shell")]));
    lines.push(Line::from(vec![Span::styled(format!("{: <12}", "r"), Style::default().fg(Color::Yellow)), Span::raw("Restart container")]));
    lines.push(Line::from(vec![Span::styled(format!("{: <12}", "s"), Style::default().fg(Color::Yellow)), Span::raw("Stop container")]));
    lines.push(Line::from(vec![Span::styled(format!("{: <12}", "t"), Style::default().fg(Color::Yellow)), Span::raw("Start container")]));
    lines.push(Line::from(vec![Span::styled(format!("{: <12}", "p"), Style::default().fg(Color::Yellow)), Span::raw("Pause container")]));
    lines.push(Line::from(vec![Span::styled(format!("{: <12}", "u"), Style::default().fg(Color::Yellow)), Span::raw("Unpause container")]));
    lines.push(Line::from(vec![Span::styled(format!("{: <12}", "d"), Style::default().fg(Color::Yellow)), Span::raw("Remove container (force)")]));
    lines.push(Line::from(vec![Span::styled(format!("{: <12}", "f"), Style::default().fg(Color::Yellow)), Span::raw("Toggle filter (all/running)")]));

    // IMAGE VIEW
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled("IMAGE VIEW", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]));
    lines.push(Line::from(vec![Span::styled(format!("{: <12}", "Up/Down"), Style::default().fg(Color::Yellow)), Span::raw("Navigate images")]));
    lines.push(Line::from(vec![Span::styled(format!("{: <12}", "Enter"), Style::default().fg(Color::Yellow)), Span::raw("Inspect image details")]));
    lines.push(Line::from(vec![Span::styled(format!("{: <12}", "s"), Style::default().fg(Color::Yellow)), Span::raw("Toggle sort (Date / Size)")]));
    lines.push(Line::from(vec![Span::styled(format!("{: <12}", "f"), Style::default().fg(Color::Yellow)), Span::raw("Toggle dangling image filter")]));
    lines.push(Line::from(vec![Span::styled(format!("{: <12}", "p"), Style::default().fg(Color::Yellow)), Span::raw("Pull new image")]));
    lines.push(Line::from(vec![Span::styled(format!("{: <12}", "d"), Style::default().fg(Color::Yellow)), Span::raw("Remove image")]));
    lines.push(Line::from(vec![Span::styled(format!("{: <12}", "D"), Style::default().fg(Color::Yellow)), Span::raw("Force remove image")]));

    let paragraph = Paragraph::new(lines)
        .scroll((scroll, 0))
        .block(Block::default().padding(ratatui::widgets::Padding::new(2, 2, 0, 1)));
    
    f.render_widget(paragraph, area);
}

fn render_wiki(f: &mut Frame<'_>, area: Rect, scroll: u16) {
    let mut lines = Vec::new();

    lines.push(Line::from(vec![
        Span::styled(" DOCKYARD WIKI ", Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD)),
    ]).alignment(Alignment::Center));

    // MANAGING CONTAINERS
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled("MANAGING CONTAINERS", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]));
    lines.push(Line::from("Use the Tab key to switch between container and image views."));
    lines.push(Line::from("Navigate with j/k or arrow keys. The list shows name, image, status, ports, and real-time CPU/memory usage."));
    lines.push(Line::from("Press Enter for detailed info (env vars, volumes, networks, labels)."));
    lines.push(Line::from("Press 'l' for logs, or 'e' for an interactive shell."));
    lines.push(Line::from("Controls: 's' (stop), 't' (start), 'r' (restart), 'p' (pause), 'u' (unpause), 'd' (remove)."));

    // MANAGING IMAGES
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled("MANAGING IMAGES", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]));
    lines.push(Line::from("Press Shift+Tab to switch to the image view. The list auto-refreshes every 30 seconds."));
    lines.push(Line::from("Press Enter or Space to inspect image details in the left pane."));
    lines.push(Line::from("Sort with 's' or filter dangling images with 'f'."));

    // PULLING & REMOVING IMAGES
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled("PULLING & REMOVING IMAGES", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]));
    lines.push(Line::from("Press 'p' in image view. Enter image name (e.g., nginx:latest)."));
    lines.push(Line::from("Progress streams in the bottom-right pane. The UI stays responsive during pull."));
    lines.push(Line::from("Press 'd' to remove (with prompt) or 'D' to force remove."));

    // HEALTH MONITORING
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled("HEALTH MONITORING", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]));
    lines.push(Line::from("Dockyard monitors container health checks for running containers. Dockyard parses the Docker health check results for real-time status."));
    lines.push(Line::from("Health status indicators:"));
    lines.push(Line::from(vec![Span::styled("- healthy:  ", Style::default().fg(Color::Green)), Span::raw("Health check is passing")]));
    lines.push(Line::from(vec![Span::styled("- unhealthy:", Style::default().fg(Color::Red)), Span::raw("Health check is failing")]));
    lines.push(Line::from(vec![Span::styled("- starting: ", Style::default().fg(Color::Yellow)), Span::raw("Health check is initializing")]));
    lines.push(Line::from(vec![Span::styled("- none:     ", Style::default().fg(Color::DarkGray)), Span::raw("No health check configured")]));
    lines.push(Line::from("The container list title shows a summary: healthy (v), starting (!), and unhealthy (x)."));

    // VISUAL FEEDBACK
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled("VISUAL FEEDBACK", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]));
    lines.push(Line::from("- Sort indicators (up/down arrows) appear in table headers."));
    lines.push(Line::from("- Stats marked as (stale) are older than 10 seconds."));
    lines.push(Line::from("- Real-time progress bars show ongoing operations like image pulls."));
    lines.push(Line::from("- Confirmation prompts appear for destructive actions."));

    // PERFORMANCE MODES
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled("PERFORMANCE MODES", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]));
    lines.push(Line::from(vec![
        Span::styled("Turbo Mode (t): ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw("Aggressive optimization for low-spec systems."),
    ]));
    lines.push(Line::from("- Only fetches stats for containers currently visible on screen."));
    lines.push(Line::from("- Switches to minimalist UI to save CPU cycles."));
    lines.push(Line::from("- Ideal for single-core servers or massive fleets."));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("Normal Mode: ", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
        Span::raw("Full visibility and detailed history for all containers."),
    ]));

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("GitHub: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled("https://github.com/905timur/dockyard", Style::default().fg(Color::DarkGray)),
    ]).alignment(Alignment::Center));

    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: true })
        .scroll((scroll, 0))
        .block(Block::default().padding(ratatui::widgets::Padding::new(2, 2, 0, 1)));
        
    f.render_widget(paragraph, area);
}
