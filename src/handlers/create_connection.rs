use crate::app::{App, FocusType};
use crate::event::Key;
use sqlx::mysql::MySqlPool;

pub async fn handler(_key: Key, app: &mut App) -> anyhow::Result<()> {
    if let Some(conn) = app.selected_connection() {
        app.pool.as_ref().unwrap().close().await;
        let pool = MySqlPool::connect(conn.database_url().as_str()).await?;
        app.pool = Some(pool);
        app.focus_type = FocusType::Dabatases(true);
    }
    Ok(())
}
