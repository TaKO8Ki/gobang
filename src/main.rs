mod app;
mod handlers;
mod ui;

use crate::app::{Database, FocusType, InputMode, Table};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
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

enum Event<I> {
    Input(I),
    Tick,
}

pub struct StatefulTable {
    state: TableState,
    headers: Vec<String>,
    items: Vec<Vec<String>>,
}

impl StatefulTable {
    fn new() -> StatefulTable {
        StatefulTable {
            state: TableState::default(),
            headers: vec![],
            items: vec![],
        }
    }
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    enable_raw_mode()?;

    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);

    let mut terminal = Terminal::new(backend)?;

    // Setup input handling
    let (tx, rx) = mpsc::channel();

    let tick_rate = Duration::from_millis(250);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            // poll for tick rate duration, if no events, sent tick event.
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));
            if event::poll(timeout).unwrap() {
                if let CEvent::Key(key) = event::read().unwrap() {
                    tx.send(Event::Input(key)).unwrap();
                }
            }
            if last_tick.elapsed() >= tick_rate {
                tx.send(Event::Tick).unwrap();
                last_tick = Instant::now();
            }
        }
    });

    let mut app = &mut app::App::default();
    let pool = MySqlPool::connect("mysql://root:@localhost:3306").await?;
    let databases = sqlx::query("show databases")
        .fetch_all(&pool)
        .await?
        .iter()
        .map(|table| table.get(0))
        .collect::<Vec<String>>();
    for db in databases {
        app.databases.push(Database::new(db, &pool).await?)
    }

    &pool.execute("use dev_payer").await?;
    let mut rows = sqlx::query("SELECT * FROM incoming_invoices").fetch(&pool);
    let mut headers: Vec<String> = vec![];
    let mut records = vec![];

    while let Some(row) = rows.try_next().await? {
        if headers.is_empty() {
            headers.extend(
                row.columns()
                    .iter()
                    .map(|col| col.name().to_string())
                    .collect::<Vec<String>>(),
            );
        }
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

    terminal.clear()?;
    let mut table = StatefulTable::new();
    table.items = records;
    table.headers = headers;

    loop {
        terminal.draw(|f| ui::draw(f, &mut app, &mut table).unwrap())?;
        match rx.recv()? {
            Event::Input(event) => match app.input_mode {
                InputMode::Normal => match event.code {
                    KeyCode::Char('e') => {
                        app.input_mode = InputMode::Editing;
                    }
                    KeyCode::Char('q') => {
                        disable_raw_mode()?;
                        execute!(
                            terminal.backend_mut(),
                            LeaveAlternateScreen,
                            DisableMouseCapture
                        )?;
                        terminal.show_cursor()?;
                        break;
                    }
                    KeyCode::Char('l') => app.focus_type = FocusType::Records(false),
                    KeyCode::Char('h') => app.focus_type = FocusType::Dabatases(false),
                    KeyCode::Char('j') => {
                        if let FocusType::Dabatases(_) = app.focus_type {
                            app.focus_type = FocusType::Tables(false)
                        }
                    }
                    KeyCode::Char('k') => {
                        if let FocusType::Tables(_) = app.focus_type {
                            app.focus_type = FocusType::Dabatases(false)
                        }
                    }
                    KeyCode::Up => match app.focus_type {
                        FocusType::Records(true) => table.previous(),
                        FocusType::Dabatases(true) => app.previous(),
                        FocusType::Tables(true) => match app.selected_database.selected() {
                            Some(index) => app.databases[index].previous(),
                            None => (),
                        },
                        _ => (),
                    },
                    KeyCode::Down => match app.focus_type {
                        FocusType::Records(true) => table.next(),
                        FocusType::Dabatases(true) => app.next(),
                        FocusType::Tables(true) => match app.selected_database.selected() {
                            Some(index) => {
                                &app.databases[index].next();
                                let db =
                                    &app.databases[app.selected_database.selected().unwrap_or(0)];
                                &pool.execute(format!("use {}", db.name).as_str()).await?;
                                let table_name = format!(
                                    "SELECT * FROM {}",
                                    &db.tables[db.selected_table.selected().unwrap_or(0)].name
                                );
                                let mut rows = sqlx::query(table_name.as_str()).fetch(&pool);
                                let mut headers: Vec<String> = vec![];
                                let mut records = vec![];

                                while let Some(row) = rows.try_next().await? {
                                    if headers.is_empty() {
                                        headers.extend(
                                            row.columns()
                                                .iter()
                                                .map(|col| col.name().to_string())
                                                .collect::<Vec<String>>(),
                                        );
                                    }
                                    let mut row_vec = vec![];
                                    for col in row.columns() {
                                        let col_name = col.name();
                                        // println!("{}", col.type_info().name());
                                        match col.type_info().clone().name() {
                                            "INT" => {
                                                let value: i32 = row.try_get(col_name).unwrap_or(0);
                                                row_vec.push(value.to_string());
                                            }
                                            "VARCHAR" => {
                                                let value: String =
                                                    row.try_get(col_name).unwrap_or("".to_string());
                                                row_vec.push(value);
                                            }
                                            _ => (),
                                        }
                                    }
                                    records.push(row_vec)
                                }

                                table.items = records;
                                table.headers = headers;
                            }
                            None => (),
                        },
                        _ => (),
                    },
                    KeyCode::Enter => match &app.focus_type {
                        FocusType::Records(false) => app.focus_type = FocusType::Records(true),
                        FocusType::Dabatases(false) => app.focus_type = FocusType::Dabatases(true),
                        FocusType::Tables(false) => app.focus_type = FocusType::Tables(true),
                        _ => (),
                    },
                    _ => {}
                },
                InputMode::Editing => match event.code {
                    KeyCode::Enter => {
                        app.messages.push(vec![app.input.drain(..).collect()]);
                    }
                    KeyCode::Char(c) => {
                        app.input.push(c);
                    }
                    KeyCode::Backspace => {
                        app.input.pop();
                    }
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                    }
                    _ => {}
                },
            },
            Event::Tick => (),
        }
    }

    Ok(())
}
