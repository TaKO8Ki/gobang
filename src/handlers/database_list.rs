use crate::app::{App, Database};
use crate::event::Key;
use crate::utils::get_databases;

pub async fn handler(_key: Key, app: &mut App) -> anyhow::Result<()> {
    app.databases = match app.selected_connection() {
        Some(conn) => match &conn.database {
            Some(database) => {
                vec![Database::new(database.clone(), app.pool.as_ref().unwrap()).await?]
            }
            None => get_databases(app.pool.as_ref().unwrap()).await?,
        },
        None => vec![],
    };
    Ok(())
}
