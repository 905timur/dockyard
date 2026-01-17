use crossterm::event::KeyCode;
use crate::app::{App, View, Focus};
use std::time::Instant;

pub async fn handle_key_events(key: KeyCode, app: &mut App, last_selection_change: &mut Instant, needs_fetch: &mut bool) -> bool {
    // 1. Handle Pull Dialog (Input)
    if app.show_pull_dialog {
        match key {
            KeyCode::Esc => app.show_pull_dialog = false,
            KeyCode::Enter => {
                if !app.pull_input.is_empty() {
                    let image = app.pull_input.clone();
                    app.start_pull_image(image);
                }
            }
            KeyCode::Backspace => {
                app.pull_input.pop();
            }
            KeyCode::Char(c) => {
                app.pull_input.push(c);
            }
            _ => {}
        }
        return false;
    }

    // 2. Handle Delete Confirmation
    if app.show_delete_confirm {
        match key {
            KeyCode::Char('y') | KeyCode::Enter => {
                let force = app.pending_delete_force;
                let _ = app.remove_current_image(force).await;
                app.show_delete_confirm = false;
            }
            KeyCode::Esc | KeyCode::Char('n') => {
                app.show_delete_confirm = false;
            }
            _ => {}
        }
        return false;
    }

    // 3. Handle Help
    if app.show_help {
        match key {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {
                app.show_help = false;
            }
            _ => {}
        }
        return false;
    }

    // 4. Global Keys
    match key {
        KeyCode::Char('?') => {
            app.show_help = true;
            return false;
        }
        KeyCode::Char('q') => return true,
        KeyCode::Char('i') => {
            if app.current_view != View::Images {
                app.current_view = View::Images;
                *needs_fetch = true;
                // Trigger details fetch for initial selection
                app.trigger_image_details();
            }
            return false;
        }
        _ => {}
    }

    // 5. View Specific Keys
    match app.current_view {
        View::Containers => {
            match key {
                KeyCode::Esc => return true, 
                KeyCode::Tab => {
                    app.focus = match app.focus {
                        Focus::ContainerList => Focus::Logs,
                        Focus::Logs => Focus::ContainerList,
                    };
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    match app.focus {
                        Focus::ContainerList => {
                            app.next();
                            *last_selection_change = Instant::now();
                            *needs_fetch = true;
                        }
                        Focus::Logs => {
                            app.auto_scroll = false;
                            let logs_len = app.selected_container_logs.read().unwrap().len();
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
                    }
                },
                KeyCode::Up | KeyCode::Char('k') => {
                    match app.focus {
                        Focus::ContainerList => {
                            app.previous();
                            *last_selection_change = Instant::now();
                            *needs_fetch = true;
                        }
                        Focus::Logs => {
                            app.auto_scroll = false;
                            let logs_len = app.selected_container_logs.read().unwrap().len();
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
                    }
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
                KeyCode::Char('p') => {
                    let _ = app.pause_container().await;
                }
                KeyCode::Char('u') => {
                    let _ = app.unpause_container().await;
                }
                KeyCode::Char('e') => {
                    if let Some(container) = app.selected_container() {
                        if container.state.to_lowercase() == "running" {
                            app.should_exec = Some(container.id);
                        }
                    }
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
                    let logs_len = app.selected_container_logs.read().unwrap().len();
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
                    let logs_len = app.selected_container_logs.read().unwrap().len();
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
        },
        View::Images => {
            match key {
                KeyCode::Esc => return true,
                KeyCode::Tab => {
                     // Switch back to containers view
                     app.current_view = View::Containers;
                     *needs_fetch = true;
                },
                KeyCode::Down | KeyCode::Char('j') => {
                    app.next_image();
                    app.trigger_image_details();
                },
                KeyCode::Up | KeyCode::Char('k') => {
                    app.previous_image();
                    app.trigger_image_details();
                },
                KeyCode::Char('p') => {
                    app.show_pull_dialog = true;
                    app.pull_input.clear();
                },
                KeyCode::Char('d') => {
                     app.show_delete_confirm = true;
                     app.pending_delete_force = false;
                },
                KeyCode::Char('D') => {
                     app.show_delete_confirm = true;
                     app.pending_delete_force = true;
                },
                KeyCode::Enter | KeyCode::Char(' ') => {
                    app.trigger_image_details();
                },
                KeyCode::Char('f') => {
                    let current = app.show_dangling.load(std::sync::atomic::Ordering::Relaxed);
                    app.show_dangling.store(!current, std::sync::atomic::Ordering::Relaxed);
                    let _ = app.refresh_images().await;
                },
                KeyCode::Char('s') => {
                     app.cycle_sort();
                     let _ = app.refresh_images().await;
                },
                _ => {}
            }
        }
    }
    false
}
