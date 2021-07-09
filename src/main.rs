mod app;
mod clipboard;
mod components;
mod event;
mod handlers;
mod ui;
mod user_config;
mod utils;

#[macro_use]
mod log;

use crate::app::App;
use crate::event::{Event, Key};
use crate::handlers::handle_app;
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

    let user_config = user_config::UserConfig::new("sample.toml").ok();

    let stdout = stdout();
    setup_terminal()?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let events = event::Events::new(250);
    let mut app = App::new(user_config.unwrap());

    terminal.clear()?;

    loop {
        terminal.draw(|f| app.draw(f).unwrap())?;
        match events.next()? {
            Event::Input(key) => {
                if key == Key::Char('q') {
                    break;
                };
                match handle_app(key, &mut app).await {
                    Ok(_) => (),
                    Err(err) => app.error.set(err.to_string()),
                }
            }
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
