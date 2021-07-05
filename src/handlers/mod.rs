pub mod connection_list;
pub mod database_list;
pub mod query;
pub mod record_table;
pub mod table_list;

use crate::app::{App, FocusBlock, Tab};
use crate::event::Key;

pub async fn handle_app(key: Key, app: &mut App) -> anyhow::Result<()> {
    match app.focus_block {
        FocusBlock::ConnectionList => connection_list::handler(key, app).await?,
        FocusBlock::DabataseList(focused) => database_list::handler(key, app, focused).await?,
        FocusBlock::TableList(focused) => table_list::handler(key, app, focused).await?,
        FocusBlock::RecordTable(focused) => record_table::handler(key, app, focused).await?,
        FocusBlock::Query(focused) => query::handler(key, app, focused).await?,
    }
    match key {
        Key::Char('d') => match app.focus_block {
            FocusBlock::Query(true) => (),
            _ => app.focus_block = FocusBlock::DabataseList(true),
        },
        Key::Char('t') => match app.focus_block {
            FocusBlock::Query(true) => (),
            _ => app.focus_block = FocusBlock::TableList(true),
        },
        Key::Char('r') => match app.focus_block {
            FocusBlock::Query(true) => (),
            _ => app.focus_block = FocusBlock::RecordTable(true),
        },
        Key::Char('e') => app.focus_block = FocusBlock::Query(true),
        Key::Char('1') => app.selected_tab = Tab::Records,
        Key::Char('2') => app.selected_tab = Tab::Structure,
        // Key::Right => app.next_tab(),
        // Key::Left => app.previous_tab(),
        Key::Esc => app.error = None,
        _ => (),
    }
    Ok(())
}
