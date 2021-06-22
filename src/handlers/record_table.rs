use crate::app::App;
use crate::event::Key;
use crate::utils::get_records;

pub async fn handler(_key: Key, app: &mut App) -> anyhow::Result<()> {
    if let Some(database) = app.selected_database() {
        if let Some(table) = app.selected_table() {
            let (headers, records) =
                get_records(database, table, app.pool.as_ref().unwrap()).await?;
            app.record_table.headers = headers;
            app.record_table.rows = records;
        }
    }
    Ok(())
}
