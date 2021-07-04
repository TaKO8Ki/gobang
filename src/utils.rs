use crate::app::{Database, Table};
use chrono::NaiveDate;
use futures::TryStreamExt;
use sqlx::mysql::{MySqlColumn, MySqlPool, MySqlRow};
use sqlx::{Column as _, Row, TypeInfo};

pub async fn get_databases(pool: &MySqlPool) -> anyhow::Result<Vec<Database>> {
    let databases = sqlx::query("SHOW DATABASES")
        .fetch_all(pool)
        .await?
        .iter()
        .map(|table| table.get(0))
        .collect::<Vec<String>>();
    let mut list = vec![];
    for db in databases {
        list.push(Database::new(db, pool).await?)
    }
    Ok(list)
}

pub async fn get_tables(database: String, pool: &MySqlPool) -> anyhow::Result<Vec<Table>> {
    let tables =
        sqlx::query_as::<_, Table>(format!("SHOW TABLE STATUS FROM `{}`", database).as_str())
            .fetch_all(pool)
            .await?;
    Ok(tables)
}

pub async fn get_records(
    database: &Database,
    table: &Table,
    pool: &MySqlPool,
) -> anyhow::Result<(Vec<String>, Vec<Vec<String>>)> {
    let query = format!("SELECT * FROM `{}`.`{}`", database.name, table.name);
    let mut rows = sqlx::query(query.as_str()).fetch(pool);
    let headers =
        sqlx::query(format!("SHOW COLUMNS FROM `{}`.`{}`", database.name, table.name).as_str())
            .fetch_all(pool)
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
    Ok((headers, records))
}

pub async fn get_columns(
    database: &Database,
    table: &Table,
    pool: &MySqlPool,
) -> anyhow::Result<(Vec<String>, Vec<Vec<String>>)> {
    let query = format!(
        "SHOW FULL COLUMNS FROM `{}`.`{}`",
        database.name, table.name
    );
    let mut rows = sqlx::query(query.as_str()).fetch(pool);
    let mut headers = vec![];
    let mut records = vec![];
    while let Some(row) = rows.try_next().await? {
        headers = row
            .columns()
            .iter()
            .map(|column| column.name().to_string())
            .collect();
        records.push(
            row.columns()
                .iter()
                .map(|col| convert_column_value_to_string(&row, col))
                .collect::<Vec<String>>(),
        )
    }
    Ok((headers, records))
}

pub fn convert_column_value_to_string(row: &MySqlRow, column: &MySqlColumn) -> String {
    let column_name = column.name();
    match column.type_info().clone().name() {
        "INT" | "DECIMAL" | "SMALLINT" => match row.try_get(column_name) {
            Ok(value) => {
                let value: i64 = value;
                value.to_string()
            }
            Err(_) => "".to_string(),
        },
        "INT UNSIGNED" => match row.try_get(column_name) {
            Ok(value) => {
                let value: u64 = value;
                value.to_string()
            }
            Err(_) => "".to_string(),
        },
        "VARCHAR" | "CHAR" | "ENUM" => row.try_get(column_name).unwrap_or_else(|_| "".to_string()),
        "DATE" => match row.try_get(column_name) {
            Ok(value) => {
                let value: NaiveDate = value;
                value.to_string()
            }
            Err(_) => "".to_string(),
        },
        "TIMESTAMP" => match row.try_get(column_name) {
            Ok(value) => {
                let value: chrono::DateTime<chrono::Utc> = value;
                value.to_string()
            }
            Err(_) => "".to_string(),
        },
        "BOOLEAN" => match row.try_get(column_name) {
            Ok(value) => {
                let value: bool = value;
                value.to_string()
            }
            Err(_) => "".to_string(),
        },
        _ => "".to_string(),
    }
}
