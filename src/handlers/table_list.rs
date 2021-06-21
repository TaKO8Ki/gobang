use crate::app::App;
use crate::event::Key;
use futures::TryStreamExt;
use sqlx::mysql::MySqlPool;
use sqlx::{Column, Executor, Row, TypeInfo};

pub async fn handler<'a>(key: Key, app: &mut App<'a>, pool: &MySqlPool) -> anyhow::Result<()> {
    match app.selected_database.selected() {
        Some(index) => {
            &app.databases[index].next();
            let db = &app.databases[app.selected_database.selected().unwrap_or(0)];
            &pool.execute(format!("use {}", db.name).as_str()).await?;
            let table_name = format!(
                "SELECT * FROM {}",
                &db.tables[db.selected_table.selected().unwrap_or(0)].name
            );
            let mut rows = sqlx::query(table_name.as_str()).fetch(pool);
            let headers = sqlx::query(
                format!(
                    "desc {}",
                    &db.tables[db.selected_table.selected().unwrap_or(0)].name
                )
                .as_str(),
            )
            .fetch_all(pool)
            .await?
            .iter()
            .map(|table| table.get(0))
            .collect::<Vec<String>>();
            let mut records = vec![];

            while let Some(row) = rows.try_next().await? {
                let mut row_vec = vec![];
                for col in row.columns() {
                    let col_name = col.name();
                    match col.type_info().clone().name() {
                        "INT" => {
                            let value: i32 = row.try_get(col_name).unwrap_or(0);
                            row_vec.push(value.to_string());
                        }
                        "VARCHAR" => {
                            let value: String = row.try_get(col_name).unwrap_or("".to_string());
                            row_vec.push(value);
                        }
                        _ => (),
                    }
                }
                records.push(row_vec)
            }

            app.record_table.rows = records;
            app.record_table.headers = headers;
        }
        None => (),
    }
    Ok(())
}
