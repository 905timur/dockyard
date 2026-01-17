use crossterm::event::{self, Event, KeyEventKind, DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;
use ratatui::{Terminal, backend::Backend};
use std::time::{Duration, Instant};
use crate::app::App;
use crate::ui::draw;
use crate::events::key_bindings::handle_key_events;
use crate::types::Result;

pub async fn run_event_loop<B: Backend + std::io::Write>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    let mut last_selection_change = Instant::now();
    let mut needs_fetch = true; 

    loop {
        // Debounced Fetch
        if needs_fetch && last_selection_change.elapsed() > Duration::from_millis(150) {
            match app.current_view {
                crate::app::View::Containers => {
                    if let Some(container) = app.selected_container() {
                        app.trigger_fetch(container.id);
                    } else {
                         // Clear if nothing selected
                        *app.selected_container_details.write().unwrap() = None;
                        app.selected_container_logs.write().unwrap().clear();
                    }
                },
                crate::app::View::Images => {
                    // Pre-fetch details if needed, but we trigger on demand mostly.
                    // However, if we want quick details, we can do it here.
                    // For now, details are triggered by Enter key as per requirements.
                }
            }
            needs_fetch = false;
        }

        // Auto-scroll logs
        if app.auto_scroll {
            let logs_len = app.selected_container_logs.read().unwrap().len();
            if logs_len > 0 {
                app.logs_state.select(Some(logs_len - 1));
            }
        }

        // Draw UI
        terminal.draw(|f| {
            draw(f, app);
        })?;

        // Poll for events
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if handle_key_events(key.code, app, &mut last_selection_change, &mut needs_fetch).await {
                        break;
                    }

                    // Check for exec request
                    if let Some(container_id) = app.should_exec.take() {
                        // Restore terminal
                        disable_raw_mode()?;
                        execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
                        terminal.show_cursor()?;
                        
                        // Run exec
                        if let Err(e) = crate::docker::exec::exec_interactive_shell(&app.docker, &container_id).await {
                            eprintln!("Exec error: {}", e);
                            tokio::time::sleep(Duration::from_secs(2)).await;
                        }
                        
                        // Setup terminal again
                        enable_raw_mode()?;
                        execute!(terminal.backend_mut(), EnterAlternateScreen, EnableMouseCapture)?;
                        terminal.hide_cursor()?;
                        terminal.clear()?;
                        
                        // Force refresh
                        app.refresh_containers().await?;
                        needs_fetch = true;
                    }
                }
            }
        }
    }
    Ok(())
}
