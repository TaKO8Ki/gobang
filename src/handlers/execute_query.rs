use crate::app::App;
use crate::event::Key;
use crate::utils::convert_column_value_to_string;
use futures::TryStreamExt;
use regex::Regex;
use sqlx::{Column, Executor, Row, TypeInfo};

pub async fn handler(_key: Key, app: &mut App) -> anyhow::Result<()> {
    let re = Regex::new(r"from (.+)").unwrap();
    let caps = re.captures(app.query.as_str()).unwrap();
    let mut rows = sqlx::query(app.query.as_str()).fetch(app.pool.as_ref().unwrap());
    let headers =
        sqlx::query(format!("desc `{}`", caps.get(1).map_or("hoge", |m| m.as_str())).as_str())
            .fetch_all(app.pool.as_ref().unwrap())
            .await?
            .iter()
            .map(|table| table.get(0))
            .collect::<Vec<String>>();
    let mut records = vec![];
    while let Some(row) = rows.try_next().await? {
        records.push(
            row.columns()
                .iter()
                .map(|col| convert_column_value_to_string(&row, col))
                .collect::<Vec<String>>(),
        )
    }
    app.record_table.headers = headers;
    app.record_table.rows = records;
    Ok(())
}
