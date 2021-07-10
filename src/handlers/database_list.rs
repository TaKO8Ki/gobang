use crate::app::{App, FocusBlock};
use crate::components::Component as _;
use crate::event::Key;
use crate::utils::{get_columns, get_records};
use database_tree::Database;

pub async fn handler(key: Key, app: &mut App) -> anyhow::Result<()> {
    match key {
        Key::Char('c')
            if matches!(
                app.databases.focus_block,
                crate::components::databases::FocusBlock::Tree
            ) =>
        {
            app.focus_block = FocusBlock::ConnectionList
        }
        Key::Right => app.focus_block = FocusBlock::Table,
        Key::Enter => {
            if let Some((table, database)) = app.databases.tree().selected_table() {
                app.focus_block = FocusBlock::Table;
                let (headers, records) = get_records(
                    &Database {
                        name: database.clone(),
                        tables: vec![],
                    },
                    &table,
                    app.pool.as_ref().unwrap(),
                )
                .await?;
                app.record_table.reset(headers, records);

                let (headers, records) = get_columns(
                    &Database {
                        name: database,
                        tables: vec![],
                    },
                    &table,
                    app.pool.as_ref().unwrap(),
                )
                .await?;
                app.structure_table.reset(headers, records);

                app.table_status
                    .update(app.record_table.rows.len() as u64, table);
            }
        }
        key => app.databases.event(key)?,
    }
    Ok(())
}
