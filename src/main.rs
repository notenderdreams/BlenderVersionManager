mod blender;
mod app;
mod network;
mod ui;

use crate::app::{App, Action};
use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    // Check for CLI arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "open" {
        let manager = blender::BlenderManager::new()?;
        if let Some(default_v) = manager.get_default_version() {
            let installed = manager.list_installed()?;
            if let Some(v) = installed.iter().find(|i| i.version == default_v) {
                println!("Launching default Blender version: {}", default_v);
                let env = manager.get_launch_env();
                blender::launch_blender(v.path.clone(), env)?;
                return Ok(());
            } else {
                println!("Error: Default version '{}' is not installed.", default_v);
                return Ok(());
            }
        } else {
            println!("Error: No default version set. Open the TUI to set one.");
            return Ok(());
        }
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = Arc::new(Mutex::new(App::new()?));
    let (tx, rx) = mpsc::channel(100);

    // Initial fetch trigger
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        let _ = tx_clone.send(Action::FetchVersions).await;
    });

    let res = app::run_app(&mut terminal, app.clone(), tx, rx).await;

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
