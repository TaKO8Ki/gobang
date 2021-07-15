pub mod connection_list;
pub mod database_list;
pub mod record_table;
pub mod structure_table;
pub mod table_filter;

use crate::app::{App, Focus};
use crate::components::tab::Tab;
use crate::components::Component as _;
use crate::event::Key;

pub async fn handle_app(key: Key, app: &mut App) -> anyhow::Result<()> {
    match key {
        Key::Esc if app.error.error.is_some() => {
            app.error.error = None;
            return Ok(());
        }
        key => app.tab.event(key)?,
    }

    match app.focus {
        Focus::ConnectionList => connection_list::handler(key, app).await?,
        Focus::DabataseList => database_list::handler(key, app).await?,
        Focus::Table => match app.tab.selected_tab {
            Tab::Records => record_table::handler(key, app).await?,
            Tab::Structure => structure_table::handler(key, app).await?,
        },
    }
    Ok(())
}
