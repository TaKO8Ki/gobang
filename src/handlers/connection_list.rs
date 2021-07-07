use crate::app::{App, FocusBlock};
use crate::event::Key;
use crate::utils::{get_databases, get_tables};
use database_tree::{Database, DatabaseTree};
use sqlx::mysql::MySqlPool;
use std::collections::BTreeSet;

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
                app.focus_block = FocusBlock::DabataseList;
            }
            if let Some(conn) = app.selected_connection() {
                match &conn.database {
                    Some(database) => {
                        app.databases.tree = DatabaseTree::new(
                            &[Database::new(
                                database.clone(),
                                get_tables(database.clone(), app.pool.as_ref().unwrap()).await?,
                            )],
                            &BTreeSet::new(),
                        )
                        .unwrap()
                    }
                    None => {
                        app.databases.tree = DatabaseTree::new(
                            get_databases(app.pool.as_ref().unwrap()).await?.as_slice(),
                            &BTreeSet::new(),
                        )
                        .unwrap()
                    }
                }
            };
        }
        _ => (),
    }
    Ok(())
}
