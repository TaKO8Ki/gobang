mod app;
mod clipboard;
mod components;
mod config;
mod database;
mod event;
mod ui;

#[macro_use]
mod log;

use crate::app::App;
use crate::event::{Event, Key};
use anyhow::Result;
use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use std::{
    io::{self, stdout},
    panic,
};
use tui::{backend::CrosstermBackend, Terminal};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    enable_raw_mode()?;

    outln!("gobang logger");

    let config = config::Config::new("sample.toml")?;

    let stdout = stdout();
    setup_terminal()?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let events = event::Events::new(250);
    let mut app = App::new(config);

    terminal.clear()?;

    loop {
        terminal.draw(|f| app.draw(f).unwrap())?;
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

fn set_panic_handlers() -> Result<()> {
    panic::set_hook(Box::new(|e| {
        eprintln!("panic: {:?}", e);
        shutdown_terminal();
    }));
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
