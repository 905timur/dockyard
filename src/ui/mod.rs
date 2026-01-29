pub mod layout;
pub mod container_list;
pub mod container_details;
pub mod logs;
pub mod help;
pub mod image_list;
pub mod image_details;

use ratatui::Frame;
use crate::app::{App, View};
use crate::ui::layout::{get_main_layout, get_right_pane_layout};
use crate::ui::container_details::{render_container_details, render_health_log_dialog};
use crate::ui::container_list::render_container_list;
use crate::ui::logs::render_container_logs;
use crate::ui::help::render_help;
use crate::ui::image_list::render_image_list;
use crate::ui::image_details::{render_image_details, render_pull_dialog, render_image_context, render_delete_confirm};

pub fn draw(f: &mut Frame<'_>, app: &mut App) {
    let area = f.area();
    
    // Split for status bar
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Min(0),
            ratatui::layout::Constraint::Length(1),
        ])
        .split(area);
    
    let main_area = chunks[0];
    let status_area = chunks[1];
    
    match app.current_view {
        View::Containers => {
            let (left, right) = get_main_layout(main_area);
            let (top_right, bottom_right) = get_right_pane_layout(right);

            render_container_details(f, left, app);
            render_container_list(f, top_right, app);
            render_container_logs(f, bottom_right, app);
            
            // Modal
            render_health_log_dialog(f, main_area, app);
        },
        View::Images => {
             let (left, right) = get_main_layout(main_area);
             let (top_right, bottom_right) = get_right_pane_layout(right);
             
             render_image_details(f, left, app);
             render_image_list(f, top_right, app);
             render_image_context(f, bottom_right, app);
             
             // Modals
             render_pull_dialog(f, main_area, app);
             render_delete_confirm(f, main_area, app);
        }
    }
    
    // Render Status Bar
    let (is_turbo, refresh_display, show_perf) = {
        let config = app.config.read().unwrap();
        (config.turbo_mode, config.refresh_rate.display(), config.show_perf_metrics)
    };

    let mode_indicator = if is_turbo {
        ratatui::text::Span::styled(" ⚡ TURBO ", ratatui::style::Style::default().fg(ratatui::style::Color::Green).bg(ratatui::style::Color::Black).add_modifier(ratatui::style::Modifier::BOLD))
    } else {
        ratatui::text::Span::styled(" 🐢 NORMAL ", ratatui::style::Style::default().fg(ratatui::style::Color::Gray).bg(ratatui::style::Color::Black))
    };

    let refresh_info = ratatui::text::Span::styled(
        format!("[{}] ", refresh_display),
        ratatui::style::Style::default().fg(ratatui::style::Color::White).bg(ratatui::style::Color::Blue)
    );

    let perf_text = if show_perf {
        let metrics = app.perf_metrics.read().unwrap();
        let mem_mb = metrics.memory_usage as f64 / 1024.0 / 1024.0;
        format!(" | CPU: {:.1}% Mem: {:.1}MB Poll: {}ms ", metrics.cpu_usage, mem_mb, metrics.poll_time_ms)
    } else {
        String::new()
    };
    
    let perf_span = ratatui::text::Span::styled(perf_text, ratatui::style::Style::default().fg(ratatui::style::Color::Yellow).bg(ratatui::style::Color::Blue));

    let help_text = match app.current_view {
        View::Containers => " Shift+Tab/v: Images | ?: Help | q: Quit | ↑/↓: Select | s: Stop | S: Start | r: Restart | d: Remove | T: Turbo | [/]: Refresh Rate",
        View::Images => " Shift+Tab/v: Containers | ?: Help | q: Quit | ↑/↓: Select | p: Pull | d: Remove | Enter: Details",
    };
    
    let status_line = ratatui::text::Line::from(vec![
        mode_indicator,
        refresh_info,
        perf_span,
        ratatui::text::Span::raw(help_text),
    ]);

    let status_bar = ratatui::widgets::Paragraph::new(status_line)
        .style(ratatui::style::Style::default().bg(ratatui::style::Color::Blue).fg(ratatui::style::Color::White));
    f.render_widget(status_bar, status_area);

    if app.show_help {
        render_help(f, area);
    }
}
