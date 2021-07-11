use crate::app::{App, FocusBlock};
use crate::components::table::RECORDS_LIMIT_PER_PAGE;
use crate::components::Component as _;
use crate::event::Key;
use crate::utils::get_records;
use database_tree::Database;

pub async fn handler(key: Key, app: &mut App) -> anyhow::Result<()> {
    match key {
        Key::Left => app.focus_block = FocusBlock::DabataseList,
        Key::Char('c') => app.focus_block = FocusBlock::ConnectionList,
        Key::Char('y') => {
            if let Some(text) = app.record_table.selected_cell() {
                app.clipboard.store(text)
            }
        }
        key => {
            app.record_table.event(key)?;
            if let Some(index) = app.record_table.state.selected() {
                if index == app.record_table.rows.len().saturating_sub(1) {
                    if let Some((table, database)) = app.databases.tree().selected_table() {
                        let (_, records) = get_records(
                            &Database {
                                name: database.clone(),
                                tables: vec![],
                            },
                            &table,
                            index as u16,
                            RECORDS_LIMIT_PER_PAGE,
                            app.pool.as_ref().unwrap(),
                        )
                        .await?;
                        if !records.is_empty() {
                            app.record_table.rows.extend(records);
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
