pub mod connection_list;
pub mod database_list;
pub mod query;
pub mod record_table;
pub mod structure_table;

use crate::app::{App, FocusBlock};
use crate::components::tab::Tab;
use crate::components::Component as _;
use crate::event::Key;

pub async fn handle_app(key: Key, app: &mut App) -> anyhow::Result<()> {
    match app.focus_block {
        FocusBlock::ConnectionList => connection_list::handler(key, app).await?,
        FocusBlock::DabataseList => database_list::handler(key, app).await?,
        FocusBlock::Table => match app.tab.selected_tab {
            Tab::Records => record_table::handler(key, app).await?,
            Tab::Structure => structure_table::handler(key, app).await?,
        },
        FocusBlock::Query => query::handler(key, app).await?,
    }
    match key {
        Key::Char('d') => match app.focus_block {
            FocusBlock::Query => (),
            _ => app.focus_block = FocusBlock::DabataseList,
        },
        Key::Char('r') => match app.focus_block {
            FocusBlock::Query => (),
            _ => app.focus_block = FocusBlock::Table,
        },
        Key::Char('e') => app.focus_block = FocusBlock::Query,
        Key::Esc => app.error = None,
        key => app.tab.event(key)?,
    }
    Ok(())
}
