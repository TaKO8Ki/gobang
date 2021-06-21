use crate::app::{App, Database};
use crate::event::Key;
use sqlx::mysql::MySqlPool;
use sqlx::Row;

pub async fn handler<'a>(key: Key, app: &mut App<'a>, pool: &MySqlPool) -> anyhow::Result<()> {
    let databases = sqlx::query("show databases")
        .fetch_all(pool)
        .await?
        .iter()
        .map(|table| table.get(0))
        .collect::<Vec<String>>();
    for db in databases {
        app.databases.push(Database::new(db, pool).await?)
    }
    Ok(())
}
