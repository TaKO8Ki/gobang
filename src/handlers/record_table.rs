use crate::app::App;
use crate::event::Key;
use sqlx::mysql::MySqlPool;

pub async fn handler(key: Key, app: &mut App, pool: &MySqlPool) -> anyhow::Result<()> {
    Ok(())
}
