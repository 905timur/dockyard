use crossterm::event::{self, Event, KeyEventKind};
use ratatui::{Terminal, backend::Backend};
use std::time::{Duration, Instant};
use crate::app::App;
use crate::ui::draw;
use crate::events::key_bindings::handle_key_events;
use crate::types::Result;

pub async fn run_event_loop<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    let mut last_selection_change = Instant::now();
    let mut needs_fetch = true; 

    loop {
        // Debounced Fetch
        if needs_fetch && last_selection_change.elapsed() > Duration::from_millis(150) {
            if let Some(container) = app.selected_container() {
                app.trigger_fetch(container.id);
            } else {
                 // Clear if nothing selected
                *app.selected_container_details.write().unwrap() = None;
                app.selected_container_logs.write().unwrap().clear();
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
                }
            }
        }
    }
    Ok(())
}
