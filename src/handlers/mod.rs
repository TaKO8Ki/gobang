pub mod create_connection;
pub mod database_list;
pub mod execute_query;
pub mod record_table;

use crate::app::{App, FocusType, InputMode};
use crate::event::Key;

pub async fn handle_app(key: Key, app: &mut App) -> anyhow::Result<()> {
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
                    Some(_) => {
                        app.record_table.column_index = 0;
                        app.previous_table();
                        record_table::handler(key, app).await?;
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
                    Some(_) => {
                        app.record_table.column_index = 0;
                        app.next_table();
                        record_table::handler(key, app).await?
                    }
                    None => (),
                },
                _ => (),
            },
            Key::Enter => match app.focus_type {
                FocusType::Connections => {
                    app.selected_database.select(Some(0));
                    app.selected_table.select(Some(0));
                    create_connection::handler(key, app).await?;
                    database_list::handler(key, app).await?;
                }
                FocusType::Records(false) => app.focus_type = FocusType::Records(true),
                FocusType::Dabatases(false) => app.focus_type = FocusType::Dabatases(true),
                FocusType::Tables(false) => app.focus_type = FocusType::Tables(true),
                _ => (),
            },
            _ => {}
        },
        InputMode::Editing => match key {
            Key::Enter => {
                app.query = app.input.drain(..).collect();
                execute_query::handler(key, app).await?;
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
