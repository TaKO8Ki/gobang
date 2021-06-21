use crate::app::{App, Database};
use crate::event::Key;
use crate::utils::get_databases;
use sqlx::mysql::MySqlPool;
use sqlx::Row;

pub async fn handler<'a>(key: Key, app: &mut App<'a>, pool: &MySqlPool) -> anyhow::Result<()> {
    for db in get_databases(pool).await? {
        app.databases.push(db)
    }
    Ok(())
}
