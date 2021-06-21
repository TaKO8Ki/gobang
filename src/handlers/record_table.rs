use crate::app::App;
use crate::event::Key;
use sqlx::mysql::MySqlPool;

pub async fn handler<'a>(key: Key, app: &mut App<'a>, pool: &MySqlPool) -> anyhow::Result<()> {
    Ok(())
}
