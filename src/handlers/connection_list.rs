use crate::app::{App, FocusBlock};
use crate::components::Component as _;
use crate::event::Key;
use crate::utils::{get_databases, get_tables};
use database_tree::{Database, DatabaseTree};
use sqlx::mysql::MySqlPool;
use std::collections::BTreeSet;

pub async fn handler(key: Key, app: &mut App) -> anyhow::Result<()> {
    match key {
        Key::Enter => {
            app.record_table.reset(vec![], vec![]);
            app.record_table.state.select(Some(0));
            if let Some(conn) = app.connections.selected_connection() {
                if let Some(pool) = app.pool.as_ref() {
                    pool.close().await;
                }
                let pool = MySqlPool::connect(conn.database_url().as_str()).await?;
                app.pool = Some(pool);
                app.focus_block = FocusBlock::DabataseList;
            }
            if let Some(conn) = app.connections.selected_connection() {
                match &conn.database {
                    Some(database) => app
                        .databases
                        .update(
                            &[Database::new(
                                database.clone(),
                                get_tables(database.clone(), app.pool.as_ref().unwrap()).await?,
                            )],
                            &BTreeSet::new(),
                        )
                        .unwrap(),
                    None => app
                        .databases
                        .update(
                            get_databases(app.pool.as_ref().unwrap()).await?.as_slice(),
                            &BTreeSet::new(),
                        )
                        .unwrap(),
                }
            };
        }
        key => app.connections.event(key)?,
    }
    Ok(())
}
