use crate::app::App;
use crate::event::Key;
use crate::utils::get_records;
use sqlx::mysql::MySqlPool;

pub async fn handler<'a>(key: Key, app: &mut App<'a>, pool: &MySqlPool) -> anyhow::Result<()> {
    if let Some(database) = app.selected_database() {
        if let Some(table) = app.selected_table() {
            let (headers, records) = get_records(database, table, pool).await?;
            app.record_table.headers = headers;
            app.record_table.rows = records;
        }
    }
    Ok(())
}
