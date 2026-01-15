pub mod layout;
pub mod container_list;
pub mod container_details;
pub mod logs;
pub mod help;

use ratatui::Frame;
use crate::app::App;
use crate::ui::layout::{get_main_layout, get_right_pane_layout};
use crate::ui::container_details::render_container_details;
use crate::ui::container_list::render_container_list;
use crate::ui::logs::render_container_logs;
use crate::ui::help::render_help;

pub fn draw(f: &mut Frame<'_>, app: &App) {
    let area = f.area();
    let (left, right) = get_main_layout(area);
    let (top_right, bottom_right) = get_right_pane_layout(right);

    render_container_details(f, left, app);
    render_container_list(f, top_right, app);
    render_container_logs(f, bottom_right, app);

    if app.show_help {
        render_help(f, area);
    }
}
