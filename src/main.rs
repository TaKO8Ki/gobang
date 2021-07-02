mod app;
mod event;
mod handlers;
mod ui;
mod user_config;
mod utils;

use crate::app::{App, FocusBlock};
use crate::event::{Event, Key};
use crate::handlers::handle_app;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::stdout;
use tui::{backend::CrosstermBackend, Terminal};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    enable_raw_mode()?;

    let user_config = user_config::UserConfig::new("sample.toml").ok();

    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let events = event::Events::new(250);

    let mut app = App {
        user_config,
        focus_block: FocusBlock::ConnectionList,
        ..App::default()
    };

    terminal.clear()?;

    loop {
        terminal.draw(|f| ui::draw(f, &mut app).unwrap())?;
        match events.next()? {
            Event::Input(key) => {
                if key == Key::Char('q') {
                    break;
                };
                match handle_app(key, &mut app).await {
                    Ok(_) => (),
                    Err(err) => app.error = Some(err.to_string()),
                }
            }
            Event::Tick => (),
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
