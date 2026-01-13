use crossterm::event::KeyCode;
use crate::app::App;
use std::time::Instant;

pub async fn handle_key_events(key: KeyCode, app: &mut App, last_selection_change: &mut Instant, needs_fetch: &mut bool) -> bool {
    // Return true if should quit
    if app.show_help {
        match key {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {
                app.show_help = false;
            }
            _ => {}
        }
    } else {
        match key {
            KeyCode::Char('?') => app.show_help = true,
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Down | KeyCode::Char('j') => {
                app.next();
                *last_selection_change = Instant::now();
                *needs_fetch = true;
            },
            KeyCode::Up | KeyCode::Char('k') => {
                app.previous();
                *last_selection_change = Instant::now();
                *needs_fetch = true;
            },
            KeyCode::Char('r') => {
                let _ = app.restart_container().await;
                let _ = app.refresh_containers().await;
            }
            KeyCode::Char('s') => {
                let _ = app.stop_container().await;
                let _ = app.refresh_containers().await;
            }
            KeyCode::Char('t') => {
                let _ = app.start_container().await;
                let _ = app.refresh_containers().await;
            }
            KeyCode::Char('d') => {
                let _ = app.remove_container().await;
            }
            KeyCode::Char('f') => {
                app.toggle_filter();
                let _ = app.refresh_containers().await;
                *needs_fetch = true;
            }
            KeyCode::Char('a') => {
                app.auto_scroll = !app.auto_scroll;
            }
            KeyCode::Char('J') => {
                app.auto_scroll = false;
                let logs_len = app.selected_container_logs.read().await.len();
                if logs_len > 0 {
                    let i = match app.logs_state.selected() {
                        Some(i) => {
                            if i >= logs_len - 1 { logs_len - 1 } else { i + 1 }
                        },
                        None => 0,
                    };
                    app.logs_state.select(Some(i));
                }
            }
            KeyCode::Char('K') => {
                app.auto_scroll = false;
                let logs_len = app.selected_container_logs.read().await.len();
                if logs_len > 0 {
                    let i = match app.logs_state.selected() {
                        Some(i) => {
                            if i == 0 { 0 } else { i - 1 }
                        },
                        None => 0,
                    };
                    app.logs_state.select(Some(i));
                }
            }
            _ => {}
        }
    }
    false
}
