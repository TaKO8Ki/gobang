mod app;
mod event;
mod handlers;
mod ui;
mod user_config;
mod utils;

use crate::app::{Database, FocusType, InputMode, Table};
use crate::event::{Event, Key};
use crate::handlers::handle_app;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::TryStreamExt;
use sqlx::mysql::MySqlPool;
use sqlx::{Column, Executor, Row, TypeInfo};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::{
    error::Error,
    io::stdout,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
use tui::{backend::CrosstermBackend, widgets::TableState, Terminal};
use user_config::UserConfig;

pub struct StatefulTable {
    state: TableState,
    headers: Vec<String>,
    items: Vec<Vec<String>>,
}

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
    let pool = MySqlPool::connect("mysql://root:@localhost:3306").await?;

    app.databases = utils::get_databases(&pool).await?;
    &pool.execute("use dev_payer").await?;
    let mut rows = sqlx::query("SELECT * FROM incoming_invoices").fetch(&pool);
    let headers = sqlx::query("desc incoming_invoices")
        .fetch_all(&pool)
        .await?
        .iter()
        .map(|table| table.get(0))
        .collect::<Vec<String>>();
    let mut records = vec![];

    while let Some(row) = rows.try_next().await? {
        let mut row_vec = vec![];
        for col in row.columns() {
            let col_name = col.name();
            match col.type_info().clone().name() {
                "INT" => {
                    let value: i32 = row.try_get(col_name).unwrap_or(0);
                    row_vec.push(value.to_string());
                }
                "VARCHAR" => {
                    let value: String = row.try_get(col_name).unwrap_or("".to_string());
                    row_vec.push(value);
                }
                _ => (),
            }
        }
        records.push(row_vec)
    }
    app.record_table.rows = records;
    app.record_table.headers = headers;

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
