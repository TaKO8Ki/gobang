use crate::app::{App, Focus};
use crate::components::table::RECORDS_LIMIT_PER_PAGE;
use crate::components::Component as _;
use crate::event::Key;
use crate::utils::get_records;
use database_tree::Database;

pub async fn handler(key: Key, app: &mut App) -> anyhow::Result<()> {
    match key {
        Key::Left => app.focus = Focus::DabataseList,
        Key::Char('c') => app.focus = Focus::ConnectionList,
        Key::Char('y') => {
            if let Some(text) = app.record_table.table.selected_cell() {
                app.clipboard.store(text)
            }
        }
        Key::Enter
            if matches!(
                app.record_table.focus,
                crate::components::record_table::Focus::Filter
            ) =>
        {
            crate::handlers::table_filter::handler(key, app).await?
        }
        key => {
            app.record_table.event(key)?;
            if app.record_table.table.eod {
                return Ok(());
            }
            if let Some(index) = app.record_table.table.state.selected() {
                if index.saturating_add(1) % RECORDS_LIMIT_PER_PAGE as usize == 0 {
                    if let Some((table, database)) = app.databases.tree().selected_table() {
                        let (_, records) = get_records(
                            &Database {
                                name: database.clone(),
                                tables: vec![],
                            },
                            &table,
                            index as u16,
                            RECORDS_LIMIT_PER_PAGE,
                            if app.record_table.filter.input.is_empty() {
                                None
                            } else {
                                Some(app.record_table.filter.input.to_string())
                            },
                            app.pool.as_ref().unwrap(),
                        )
                        .await?;
                        if !records.is_empty() {
                            app.record_table.table.rows.extend(records);
                        } else {
                            app.record_table.table.end()
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
