mod app;
mod event;
mod handlers;
mod ui;
mod user_config;
mod utils;

use crate::app::FocusType;
use crate::event::{Event, Key};
use crate::handlers::handle_app;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use sqlx::mysql::MySqlPool;
use std::io::stdout;
use tui::{backend::CrosstermBackend, Terminal};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    enable_raw_mode()?;

    let config = user_config::UserConfig::new("sample.toml").unwrap();

    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let events = event::Events::new(250);

    let mut app = &mut app::App::default();
    app.user_config = Some(config);
    let conn = &app.user_config.as_ref().unwrap().conn.get(0).unwrap();
    let pool = MySqlPool::connect(
        format!(
            "mysql://{user}:@{host}:{port}",
            user = conn.user,
            host = conn.host,
            port = conn.port
        )
        .as_str(),
    )
    .await?;

    app.databases = utils::get_databases(&pool).await?;
    let (headers, records) = utils::get_records(
        app.databases.first().unwrap(),
        app.databases.first().unwrap().tables.first().unwrap(),
        &pool,
    )
    .await?;
    app.record_table.rows = records;
    app.record_table.headers = headers;
    app.selected_database.select(Some(0));
    app.focus_type = FocusType::Connections;

    terminal.clear()?;

    loop {
        terminal.draw(|f| ui::draw(f, &mut app).unwrap())?;
        match events.next()? {
            Event::Input(key) => {
                if key == Key::Char('q') {
                    break;
                };
                handle_app(key, app, &pool).await?
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
