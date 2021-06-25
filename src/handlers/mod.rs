pub mod connection_list;
pub mod database_list;
pub mod query;
pub mod record_table;
pub mod table_list;

use crate::app::{App, FocusBlock, InputMode};
use crate::event::Key;

pub async fn handle_app(key: Key, app: &mut App) -> anyhow::Result<()> {
    match app.input_mode {
        InputMode::Normal => {
            match app.focus_type {
                FocusBlock::ConnectionList => connection_list::handler(key, app).await?,
                FocusBlock::DabataseList(focused) => {
                    database_list::handler(key, app, focused).await?
                }
                FocusBlock::TableList(focused) => table_list::handler(key, app, focused).await?,
                FocusBlock::RecordTable(focused) => {
                    record_table::handler(key, app, focused).await?
                }
            }
            match key {
                Key::Char('e') => app.input_mode = InputMode::Editing,
                _ => (),
            }
        }
        InputMode::Editing => match key {
            Key::Enter => query::handler(key, app).await?,
            Key::Char(c) => app.input.push(c),
            Key::Backspace => {
                app.input.pop();
            }
            Key::Esc => app.input_mode = InputMode::Normal,
            _ => {}
        },
    }
    Ok(())
}
