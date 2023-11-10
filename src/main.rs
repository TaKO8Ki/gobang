mod app;
mod cli;
mod clipboard;
mod components;
mod config;
mod database;
mod event;
mod ui;
mod version;

#[macro_use]
mod log;

use std::io;

use anyhow::Result;
use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use tui::{backend::CrosstermBackend, Terminal};

use crate::{
    app::App,
    event::{Event, Key},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let value = crate::cli::parse();
    let config = config::Config::new(&value.config)?;

    setup_terminal()?;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    let events = event::Events::new(250);
    let mut app = App::new(config.clone());

    terminal.clear()?;

    loop {
        terminal.draw(|f| {
            if let Err(err) = app.draw(f) {
                outln!(config#Error, "error: {}", err.to_string());
                std::process::exit(1);
            }
        })?;
        match events.next()? {
            Event::Input(key) => match app.event(key).await {
                Ok(state) => {
                    if !state.is_consumed()
                        && (key == app.config.key_config.quit || key == app.config.key_config.exit)
                    {
                        break;
                    }
                }
                Err(err) => app.error.set(err.to_string())?,
            },
            Event::Tick => (),
        }
    }

    shutdown_terminal();
    terminal.show_cursor()?;

    Ok(())
}

fn setup_terminal() -> Result<()> {
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    Ok(())
}

fn shutdown_terminal() {
    let leave_screen = io::stdout().execute(LeaveAlternateScreen).map(|_f| ());

    if let Err(e) = leave_screen {
        eprintln!("leave_screen failed:\n{}", e);
    }

    let leave_raw_mode = disable_raw_mode();

    if let Err(e) = leave_raw_mode {
        eprintln!("leave_raw_mode failed:\n{}", e);
    }
}
