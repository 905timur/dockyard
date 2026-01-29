pub mod app;
pub mod config;
pub mod docker;
pub mod events;
pub mod types;
pub mod ui;

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::env;

use crate::app::App;
use crate::events::handler::run_event_loop;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse args
    let args: Vec<String> = env::args().collect();
    let mut stats_interval = 3;
    for i in 0..args.len() {
        if args[i] == "--stats-interval" && i + 1 < args.len() {
             if let Ok(val) = args[i+1].parse::<u64>() {
                 stats_interval = val.max(1).min(10); // Clamp to 1-10s range per requirements
             }
        }
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize app
    let mut app = App::new(stats_interval).await?;

    // Run event loop
    let res = run_event_loop(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}
