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
use crate::ui::container_details::render_container_details;
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
    let status_text = match app.current_view {
        View::Containers => " Tab: Images | ?: Help | q: Quit | ↑/↓: Select | s: Stop | t: Start | r: Restart | d: Remove",
        View::Images => " Tab: Containers | ?: Help | q: Quit | ↑/↓: Select | p: Pull | d: Remove | Enter: Details",
    };
    let status_bar = ratatui::widgets::Paragraph::new(status_text)
        .style(ratatui::style::Style::default().bg(ratatui::style::Color::Blue).fg(ratatui::style::Color::White));
    f.render_widget(status_bar, status_area);

    if app.show_help {
        render_help(f, area);
    }
}
