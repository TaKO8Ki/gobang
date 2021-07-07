pub mod connection_list;
pub mod database_list;
pub mod query;
pub mod record_table;

use crate::app::{App, FocusBlock, Tab};
use crate::event::Key;

pub async fn handle_app(key: Key, app: &mut App) -> anyhow::Result<()> {
    match app.focus_block {
        FocusBlock::ConnectionList => connection_list::handler(key, app).await?,
        FocusBlock::DabataseList => database_list::handler(key, app).await?,
        FocusBlock::RecordTable => record_table::handler(key, app).await?,
        FocusBlock::Query => query::handler(key, app).await?,
    }
    match key {
        Key::Char('d') => match app.focus_block {
            FocusBlock::Query => (),
            _ => app.focus_block = FocusBlock::DabataseList,
        },
        Key::Char('r') => match app.focus_block {
            FocusBlock::Query => (),
            _ => app.focus_block = FocusBlock::RecordTable,
        },
        Key::Char('e') => app.focus_block = FocusBlock::Query,
        Key::Char('1') => app.selected_tab = Tab::Records,
        Key::Char('2') => app.selected_tab = Tab::Structure,
        Key::Esc => app.error = None,
        _ => (),
    }
    app.databases.focused = matches!(app.focus_block, FocusBlock::DabataseList);
    Ok(())
}
