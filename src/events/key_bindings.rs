use crossterm::event::KeyCode;
use crate::app::{App, View, Focus};
use std::time::Instant;

pub async fn handle_key_events(key: KeyCode, app: &mut App, last_selection_change: &mut Instant, needs_fetch: &mut bool) -> bool {
    // 0. Handle Health Log Dialog
    if app.show_health_log_dialog {
        match key {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('E') => {
                app.show_health_log_dialog = false;
            }
            _ => {}
        }
        return false;
    }

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
                app.current_help_tab = crate::types::HelpTab::Keybindings;
                app.help_scroll = 0; // Reset scroll
            }
            KeyCode::Tab => {
                app.current_help_tab = match app.current_help_tab {
                    crate::types::HelpTab::Keybindings => crate::types::HelpTab::Wiki,
                    crate::types::HelpTab::Wiki => crate::types::HelpTab::Keybindings,
                };
                app.help_scroll = 0; // Reset scroll on tab switch
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.help_scroll = app.help_scroll.saturating_add(1);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                app.help_scroll = app.help_scroll.saturating_sub(1);
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
        KeyCode::BackTab | KeyCode::Char('v') => {
            if app.current_view == View::Containers {
                app.current_view = View::Images;
                // Trigger details fetch for initial selection if switching to images
                app.trigger_image_details();
            } else {
                app.current_view = View::Containers;
            }
            *needs_fetch = true;
            return false;
        }
        KeyCode::Char('t') | KeyCode::Char('T') => {
            {
                let mut config = app.config.write().unwrap();
                config.turbo_mode = !config.turbo_mode;
            }
            app.apply_turbo_preset();
            app.save_config();
            *needs_fetch = true;
            return false;
        }
        KeyCode::Char('[') => {
            {
                let mut config = app.config.write().unwrap();
                let is_turbo = config.turbo_mode;
                config.refresh_rate.decrease(is_turbo);
            }
            app.save_config();
            return false;
        }
        KeyCode::Char(']') => {
            {
                let mut config = app.config.write().unwrap();
                let is_turbo = config.turbo_mode;
                config.refresh_rate.increase(is_turbo);
            }
            app.save_config();
            return false;
        }
        KeyCode::Char('m') | KeyCode::Char('M') => {
            {
                let mut config = app.config.write().unwrap();
                config.stats_view.toggle();
            }
            app.save_config();
            *needs_fetch = true;
            return false;
        }
        KeyCode::Char('R') => {
            let _ = app.refresh_containers().await;
            if app.current_view == View::Images {
                let _ = app.refresh_images().await;
            }
            *needs_fetch = true;
            return false;
        }
        KeyCode::Char('P') => {
            {
                let mut config = app.config.write().unwrap();
                config.show_perf_metrics = !config.show_perf_metrics;
            }
            app.save_config();
            return false;
        }
        KeyCode::Char('1') => {
            // Preset 1: Max Performance
            {
                let mut config = app.config.write().unwrap();
                config.turbo_mode = true;
                config.refresh_rate = crate::types::RefreshRate::Manual;
                config.stats_view = crate::types::StatsView::Minimal;
                config.poll_strategy = crate::types::PollStrategy::VisibleOnly;
            }
            app.save_config();
            *needs_fetch = true;
            return false;
        }
        KeyCode::Char('2') => {
            // Preset 2: Balanced
            {
                let mut config = app.config.write().unwrap();
                config.turbo_mode = false;
                config.refresh_rate = crate::types::RefreshRate::Interval(std::time::Duration::from_secs(5));
                config.stats_view = crate::types::StatsView::Minimal;
                config.poll_strategy = crate::types::PollStrategy::AllContainers;
            }
            app.save_config();
            *needs_fetch = true;
            return false;
        }
        KeyCode::Char('3') => {
            // Preset 3: Full Detail
            {
                let mut config = app.config.write().unwrap();
                config.turbo_mode = false;
                config.refresh_rate = crate::types::RefreshRate::Interval(std::time::Duration::from_secs(1));
                config.stats_view = crate::types::StatsView::Detailed;
                config.poll_strategy = crate::types::PollStrategy::AllContainers;
            }
            app.save_config();
            *needs_fetch = true;
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
                KeyCode::Char('S') => {
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
                KeyCode::Char('h') => {
                    app.toggle_health_filter();
                    *needs_fetch = true;
                }
                KeyCode::Char('H') => {
                    app.cycle_container_sort();
                    *needs_fetch = true;
                }
                KeyCode::Char('E') => {
                    if let Some(c) = app.selected_container() {
                        let health = app.container_health.read().unwrap();
                        if let Some(h) = health.get(&c.id) {
                            if let Some(output) = &h.last_check_output {
                                app.health_log_content = output.clone();
                                app.show_health_log_dialog = true;
                            } else {
                                app.health_log_content = "No output available.".to_string();
                                app.show_health_log_dialog = true;
                            }
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
