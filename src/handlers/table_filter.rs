use crate::app::App;
use crate::components::table::RECORDS_LIMIT_PER_PAGE;
use crate::event::Key;
use crate::utils::get_records;
use database_tree::Database;

pub async fn handler(key: Key, app: &mut App) -> anyhow::Result<()> {
    match key {
        Key::Enter => {
            app.record_table.focus = crate::components::record_table::Focus::Table;
            let filter_input = app.record_table.filter.input.to_string();

            if app.record_table.filter.input.is_empty() {
                return Ok(());
            }

            if let Some((table, database)) = app.databases.tree().selected_table() {
                let (headers, records) = get_records(
                    &Database {
                        name: database.clone(),
                        tables: vec![],
                    },
                    &table,
                    0,
                    RECORDS_LIMIT_PER_PAGE,
                    Some(filter_input),
                    app.pool.as_ref().unwrap(),
                )
                .await?;
                app.record_table.update(records, headers);
            }
        }
        _ => (),
    }
    Ok(())
}
