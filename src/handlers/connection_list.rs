use crate::app::{App, Database, FocusBlock};
use crate::event::Key;
use crate::utils::get_databases;
use sqlx::mysql::MySqlPool;

pub async fn handler(key: Key, app: &mut App) -> anyhow::Result<()> {
    match key {
        Key::Char('j') => app.next_connection(),
        Key::Char('k') => app.previous_connection(),
        Key::Enter => {
            app.selected_database.select(Some(0));
            app.selected_table.select(Some(0));
            app.record_table.state.select(Some(0));
            if let Some(conn) = app.selected_connection() {
                if let Some(pool) = app.pool.as_ref() {
                    pool.close().await;
                }
                let pool = MySqlPool::connect(conn.database_url().as_str()).await?;
                app.pool = Some(pool);
                app.focus_block = FocusBlock::DabataseList(false);
            }
            app.databases = match app.selected_connection() {
                Some(conn) => match &conn.database {
                    Some(database) => {
                        vec![Database::new(database.clone(), app.pool.as_ref().unwrap()).await?]
                    }
                    None => get_databases(app.pool.as_ref().unwrap()).await?,
                },
                None => vec![],
            };
        }
        _ => (),
    }
    Ok(())
}
