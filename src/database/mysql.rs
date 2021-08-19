use super::{Pool, RECORDS_LIMIT_PER_PAGE};
use async_trait::async_trait;
use chrono::NaiveDate;
use database_tree::{Child, Database, Table};
use futures::TryStreamExt;
use sqlx::mysql::{MySqlColumn, MySqlPool as MPool, MySqlRow};
use sqlx::{Column as _, Row as _, TypeInfo as _};

pub struct MySqlPool {
    pool: MPool,
}

impl MySqlPool {
    pub async fn new(database_url: &str) -> anyhow::Result<Self> {
        Ok(Self {
            pool: MPool::connect(database_url).await?,
        })
    }
}

#[async_trait]
impl Pool for MySqlPool {
    async fn get_databases(&self) -> anyhow::Result<Vec<Database>> {
        let databases = sqlx::query("SHOW DATABASES")
            .fetch_all(&self.pool)
            .await?
            .iter()
            .map(|table| table.get(0))
            .collect::<Vec<String>>();
        let mut list = vec![];
        for db in databases {
            list.push(Database::new(
                db.clone(),
                self.get_tables(db.clone()).await?,
            ))
        }
        Ok(list)
    }

    async fn get_tables(&self, database: String) -> anyhow::Result<Vec<Child>> {
        let tables =
            sqlx::query_as::<_, Table>(format!("SHOW TABLE STATUS FROM `{}`", database).as_str())
                .fetch_all(&self.pool)
                .await?;
        Ok(tables.into_iter().map(|table| table.into()).collect())
    }

    async fn get_records(
        &self,
        database: &Database,
        table: &Table,
        page: u16,
        filter: Option<String>,
    ) -> anyhow::Result<(Vec<String>, Vec<Vec<String>>)> {
        let query = if let Some(filter) = filter {
            format!(
                "SELECT * FROM `{database}`.`{table}` WHERE {filter} LIMIT {page}, {limit}",
                database = database.name,
                table = table.name,
                filter = filter,
                page = page,
                limit = RECORDS_LIMIT_PER_PAGE
            )
        } else {
            format!(
                "SELECT * FROM `{}`.`{}` limit {page}, {limit}",
                database.name,
                table.name,
                page = page,
                limit = RECORDS_LIMIT_PER_PAGE
            )
        };
        let mut rows = sqlx::query(query.as_str()).fetch(&self.pool);
        let mut headers = vec![];
        let mut records = vec![];
        while let Some(row) = rows.try_next().await? {
            headers = row
                .columns()
                .iter()
                .map(|column| column.name().to_string())
                .collect();
            let mut new_row = vec![];
            for column in row.columns() {
                new_row.push(convert_column_value_to_string(&row, column)?)
            }
            records.push(new_row)
        }
        Ok((headers, records))
    }

    async fn get_columns(
        &self,
        database: &Database,
        table: &Table,
    ) -> anyhow::Result<(Vec<String>, Vec<Vec<String>>)> {
        let query = format!(
            "SHOW FULL COLUMNS FROM `{}`.`{}`",
            database.name, table.name
        );
        let mut rows = sqlx::query(query.as_str()).fetch(&self.pool);
        let mut headers = vec![];
        let mut records = vec![];
        while let Some(row) = rows.try_next().await? {
            headers = row
                .columns()
                .iter()
                .map(|column| column.name().to_string())
                .collect();
            let mut new_row = vec![];
            for column in row.columns() {
                new_row.push(convert_column_value_to_string(&row, column)?)
            }
            records.push(new_row)
        }
        Ok((headers, records))
    }

    async fn close(&self) {
        self.pool.close().await;
    }
}

fn convert_column_value_to_string(row: &MySqlRow, column: &MySqlColumn) -> anyhow::Result<String> {
    let column_name = column.name();
    if let Ok(value) = row.try_get(column_name) {
        let value: Option<String> = value;
        return Ok(value.map_or("NULL".to_string(), |v| v.to_string()));
    }
    if let Ok(value) = row.try_get(column_name) {
        let value: Option<&str> = value;
        return Ok(value.map_or("NULL".to_string(), |v| v.to_string()));
    }
    if let Ok(value) = row.try_get(column_name) {
        let value: Option<i8> = value;
        return Ok(value.map_or("NULL".to_string(), |v| v.to_string()));
    }
    if let Ok(value) = row.try_get(column_name) {
        let value: Option<i32> = value;
        return Ok(value.map_or("NULL".to_string(), |v| v.to_string()));
    }
    if let Ok(value) = row.try_get(column_name) {
        let value: Option<i64> = value;
        return Ok(value.map_or("NULL".to_string(), |v| v.to_string()));
    }
    if let Ok(value) = row.try_get(column_name) {
        let value: Option<f32> = value;
        return Ok(value.map_or("NULL".to_string(), |v| v.to_string()));
    }
    if let Ok(value) = row.try_get(column_name) {
        let value: Option<rust_decimal::Decimal> = value;
        return Ok(value.map_or("NULL".to_string(), |v| v.to_string()));
    }
    if let Ok(value) = row.try_get(column_name) {
        let value: Option<u8> = value;
        return Ok(value.map_or("NULL".to_string(), |v| v.to_string()));
    }
    if let Ok(value) = row.try_get(column_name) {
        let value: Option<u16> = value;
        return Ok(value.map_or("NULL".to_string(), |v| v.to_string()));
    }
    if let Ok(value) = row.try_get(column_name) {
        let value: Option<u32> = value;
        return Ok(value.map_or("NULL".to_string(), |v| v.to_string()));
    }
    if let Ok(value) = row.try_get(column_name) {
        let value: Option<u64> = value;
        return Ok(value.map_or("NULL".to_string(), |v| v.to_string()));
    }
    if let Ok(value) = row.try_get(column_name) {
        let value: Option<NaiveDate> = value;
        return Ok(value.map_or("NULL".to_string(), |v| v.to_string()));
    }
    if let Ok(value) = row.try_get(column_name) {
        let value: Option<chrono::DateTime<chrono::Utc>> = value;
        return Ok(value.map_or("NULL".to_string(), |v| v.to_string()));
    }
    if let Ok(value) = row.try_get(column_name) {
        let value: Option<bool> = value;
        return Ok(value.map_or("NULL".to_string(), |v| v.to_string()));
    }
    Err(anyhow::anyhow!(
        "column type not implemented: `{}` {}",
        column_name,
        column.type_info().clone().name()
    ))
}
