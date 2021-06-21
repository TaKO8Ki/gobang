pub mod database_list;
pub mod record_table;
pub mod table_list;

use crate::app::{App, FocusType, InputMode};
use crate::event::Key;
use futures::TryStreamExt;
use sqlx::mysql::MySqlPool;
use sqlx::{Column, Executor, Row, TypeInfo};

pub async fn handle_app<'a>(key: Key, app: &mut App<'a>, pool: &MySqlPool) -> anyhow::Result<()> {
    match app.input_mode {
        InputMode::Normal => match key {
            Key::Char('e') => {
                app.input_mode = InputMode::Editing;
            }
            Key::Char('c') => {
                app.focus_type = FocusType::Connections;
            }
            Key::Char('l') => app.focus_type = FocusType::Records(false),
            Key::Char('h') => app.focus_type = FocusType::Tables(false),
            Key::Char('j') => {
                if let FocusType::Dabatases(_) = app.focus_type {
                    app.focus_type = FocusType::Tables(false)
                }
            }
            Key::Char('k') => {
                if let FocusType::Tables(_) = app.focus_type {
                    app.focus_type = FocusType::Dabatases(false)
                }
            }
            Key::Right => match app.focus_type {
                FocusType::Records(true) => app.record_table.next_column(),
                _ => (),
            },
            Key::Left => match app.focus_type {
                FocusType::Records(true) => app.record_table.previous_column(),
                _ => (),
            },
            Key::Up => match app.focus_type {
                FocusType::Connections => app.previous_connection(),
                FocusType::Records(true) => app.record_table.previous(),
                FocusType::Dabatases(true) => app.previous_database(),
                FocusType::Tables(true) => match app.selected_database.selected() {
                    Some(index) => {
                        app.record_table.column_index = 0;
                        app.databases[index].previous();
                        let db = &app.databases[app.selected_database.selected().unwrap_or(0)];
                        let (headers, records) = crate::utils::get_records(
                            db,
                            &db.tables[db.selected_table.selected().unwrap()],
                            &pool,
                        )
                        .await?;
                        app.record_table.rows = records;
                        app.record_table.headers = headers;
                    }
                    None => (),
                },
                _ => (),
            },
            Key::Down => match app.focus_type {
                FocusType::Connections => app.next_connection(),
                FocusType::Records(true) => app.record_table.next(),
                FocusType::Dabatases(true) => app.next_database(),
                FocusType::Tables(true) => match app.selected_database.selected() {
                    Some(index) => {
                        app.record_table.column_index = 0;
                        &app.databases[index].next();
                        let db = &app.databases[app.selected_database.selected().unwrap_or(0)];
                        let (headers, records) = crate::utils::get_records(
                            db,
                            &db.tables[db.selected_table.selected().unwrap()],
                            &pool,
                        )
                        .await?;
                        app.record_table.rows = records;
                        app.record_table.headers = headers;
                    }
                    None => (),
                },
                _ => (),
            },
            Key::Enter => match &app.focus_type {
                FocusType::Connections => app.focus_type = FocusType::Dabatases(true),
                FocusType::Records(false) => app.focus_type = FocusType::Records(true),
                FocusType::Dabatases(false) => app.focus_type = FocusType::Dabatases(true),
                FocusType::Tables(false) => app.focus_type = FocusType::Tables(true),
                _ => (),
            },
            _ => {}
        },
        InputMode::Editing => match key {
            Key::Enter => {
                app.messages.push(vec![app.input.drain(..).collect()]);
            }
            Key::Char(c) => {
                app.input.push(c);
            }
            Key::Backspace => {
                app.input.pop();
            }
            Key::Esc => {
                app.input_mode = InputMode::Normal;
            }
            _ => {}
        },
    }
    Ok(())
}
