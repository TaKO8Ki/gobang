use crate::app::{App, Focus};
use crate::components::table::RECORDS_LIMIT_PER_PAGE;
use crate::components::{Component as _, RecordTableComponent, TableComponent};
use crate::event::Key;
use crate::utils::{get_columns, get_records};
use database_tree::Database;

pub async fn handler(key: Key, app: &mut App) -> anyhow::Result<()> {
    match key {
        Key::Char('c') if app.databases.tree_focused() => app.focus = Focus::ConnectionList,
        Key::Right if app.databases.tree_focused() => app.focus = Focus::Table,
        Key::Enter if app.databases.tree_focused() => {
            if let Some((table, database)) = app.databases.tree().selected_table() {
                app.focus = Focus::Table;
                let (headers, records) = get_records(
                    &Database {
                        name: database.clone(),
                        tables: vec![],
                    },
                    &table,
                    0,
                    RECORDS_LIMIT_PER_PAGE,
                    None,
                    app.pool.as_ref().unwrap(),
                )
                .await?;
                app.record_table = RecordTableComponent::new(records, headers);
                app.record_table.set_table(table.name.to_string());

                let (headers, records) = get_columns(
                    &Database {
                        name: database,
                        tables: vec![],
                    },
                    &table,
                    app.pool.as_ref().unwrap(),
                )
                .await?;
                app.structure_table = TableComponent::new(records, headers);

                app.table_status
                    .update(app.record_table.len() as u64, table);
            }
        }
        key => app.databases.event(key)?,
    }
    Ok(())
}
