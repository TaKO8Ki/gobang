use crate::app::{Database, Table};
use chrono::{DateTime, NaiveDate, NaiveDateTime};
use futures::TryStreamExt;
use sqlx::mysql::MySqlPool;
use sqlx::{Column, Executor, Row, TypeInfo};

pub async fn get_databases(pool: &MySqlPool) -> anyhow::Result<Vec<Database>> {
    let databases = sqlx::query("show databases")
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
    let tables = sqlx::query(format!("show tables from `{}`", database).as_str())
        .fetch_all(pool)
        .await?
        .iter()
        .map(|table| Table { name: table.get(0) })
        .collect::<Vec<Table>>();
    Ok(tables)
}

pub async fn get_records(
    database: &Database,
    table: &Table,
    pool: &MySqlPool,
) -> anyhow::Result<(Vec<String>, Vec<Vec<String>>)> {
    &pool
        .execute(format!("use `{}`", database.name).as_str())
        .await?;
    let table_name = format!("SELECT * FROM `{}`", table.name);
    let mut rows = sqlx::query(table_name.as_str()).fetch(pool);
    let headers = sqlx::query(format!("desc `{}`", table.name).as_str())
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
                "INT" => match row.try_get(col_name) {
                    Ok(value) => {
                        let value: i64 = value;
                        row_vec.push(value.to_string())
                    }
                    Err(_) => row_vec.push("".to_string()),
                },
                "INT UNSIGNED" => match row.try_get(col_name) {
                    Ok(value) => {
                        let value: u64 = value;
                        row_vec.push(value.to_string())
                    }
                    Err(_) => row_vec.push("".to_string()),
                },
                "VARCHAR" => {
                    let value: String = row.try_get(col_name).unwrap_or("".to_string());
                    row_vec.push(value);
                }
                "DATE" => match row.try_get(col_name) {
                    Ok(value) => {
                        let value: NaiveDate = value;
                        row_vec.push(value.to_string())
                    }
                    Err(_) => row_vec.push("".to_string()),
                },
                "TIMESTAMP" => match row.try_get(col_name) {
                    Ok(value) => {
                        let value: chrono::DateTime<chrono::Utc> = value;
                        row_vec.push(value.to_string())
                    }
                    Err(_) => row_vec.push("".to_string()),
                },
                "BOOLEAN" => match row.try_get(col_name) {
                    Ok(value) => {
                        let value: bool = value;
                        row_vec.push(value.to_string())
                    }
                    Err(_) => row_vec.push("".to_string()),
                },
                "ENUM" => {
                    let value: String = row.try_get(col_name).unwrap_or("".to_string());
                    row_vec.push(value);
                }
                _ => (),
            }
        }
        records.push(row_vec)
    }
    Ok((headers, records))
}
