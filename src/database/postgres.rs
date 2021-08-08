use super::{Pool, RECORDS_LIMIT_PER_PAGE};
use async_trait::async_trait;
use chrono::NaiveDate;
use database_tree::{Child, Database, Schema, Table};
use futures::TryStreamExt;
use itertools::Itertools;
use sqlx::postgres::{PgColumn, PgPool, PgRow};
use sqlx::{Column as _, Row as _, TypeInfo as _};

pub struct PostgresPool {
    pool: PgPool,
}

impl PostgresPool {
    pub async fn new(database_url: &str) -> anyhow::Result<Self> {
        Ok(Self {
            pool: PgPool::connect(database_url).await?,
        })
    }
}

#[async_trait]
impl Pool for PostgresPool {
    async fn get_databases(&self) -> anyhow::Result<Vec<Database>> {
        let databases = sqlx::query("SELECT datname FROM pg_database")
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
        let mut rows =
            sqlx::query("SELECT * FROM information_schema.tables WHERE table_catalog = $1")
                .bind(database)
                .fetch(&self.pool);
        let mut tables = Vec::new();
        while let Some(row) = rows.try_next().await? {
            tables.push(Table {
                name: row.get("table_name"),
                create_time: None,
                update_time: None,
                engine: None,
                schema: row.get("table_schema"),
            })
        }
        let mut schemas = vec![];
        for (key, group) in &tables
            .iter()
            .sorted_by(|a, b| Ord::cmp(&b.schema, &a.schema))
            .group_by(|t| t.schema.as_ref())
        {
            if let Some(key) = key {
                schemas.push(
                    Schema {
                        name: key.to_string(),
                        tables: group.cloned().collect(),
                    }
                    .into(),
                )
            }
        }
        Ok(schemas)
    }

    async fn get_records(
        &self,
        database: &Database,
        table: &Table,
        page: u16,
        filter: Option<String>,
    ) -> anyhow::Result<(Vec<String>, Vec<Vec<String>>)> {
        let query = if let Some(filter) = filter.as_ref() {
            format!(
                r#"SELECT * FROM "{database}""{table_schema}"."{table}" WHERE {filter} LIMIT {page}, {limit}"#,
                database = database.name,
                table = table.name,
                filter = filter,
                table_schema = table.schema.clone().unwrap_or_else(|| "public".to_string()),
                page = page,
                limit = RECORDS_LIMIT_PER_PAGE
            )
        } else {
            format!(
                r#"SELECT * FROM "{database}"."{table_schema}"."{table}" limit {limit} offset {page}"#,
                database = database.name,
                table = table.name,
                table_schema = table.schema.clone().unwrap_or_else(|| "public".to_string()),
                page = page,
                limit = RECORDS_LIMIT_PER_PAGE
            )
        };
        let mut rows = sqlx::query(query.as_str()).fetch(&self.pool);
        let mut headers = vec![];
        let mut records = vec![];
        let mut json_records = None;
        while let Some(row) = rows.try_next().await? {
            headers = row
                .columns()
                .iter()
                .map(|column| column.name().to_string())
                .collect();
            let mut new_row = vec![];
            for column in row.columns() {
                match convert_column_value_to_string(&row, column) {
                    Ok(v) => new_row.push(v),
                    Err(_) => {
                        if json_records.is_none() {
                            json_records = Some(
                                self.get_json_records(database, table, page, filter.clone())
                                    .await?,
                            );
                        }
                        if let Some(json_records) = &json_records {
                            match json_records
                                .get(records.len())
                                .unwrap()
                                .get(column.name())
                                .unwrap()
                            {
                                serde_json::Value::String(v) => new_row.push(v.to_string()),
                                serde_json::Value::Null => new_row.push("NULL".to_string()),
                                serde_json::Value::Array(v) => {
                                    new_row.push(v.iter().map(|v| v.to_string()).join(","))
                                }
                                _ => (),
                            }
                        }
                    }
                }
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
        let table_schema = table
            .schema
            .as_ref()
            .map_or("public", |schema| schema.as_str());
        let mut rows = sqlx::query(
            "SELECT * FROM information_schema.columns WHERE table_catalog = $1 AND table_schema = $2 AND table_name = $3"
        )
        .bind(&database.name).bind(table_schema).bind(&table.name)
        .fetch(&self.pool);
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

impl PostgresPool {
    async fn get_json_records(
        &self,
        database: &Database,
        table: &Table,
        page: u16,
        filter: Option<String>,
    ) -> anyhow::Result<Vec<serde_json::Value>> {
        let query = if let Some(filter) = filter {
            format!(
                r#"SELECT to_json({table}.*) FROM "{database}""{table_schema}"."{table}" WHERE {filter} LIMIT {page}, {limit}"#,
                database = database.name,
                table = table.name,
                filter = filter,
                table_schema = table.schema.clone().unwrap_or_else(|| "public".to_string()),
                page = page,
                limit = RECORDS_LIMIT_PER_PAGE
            )
        } else {
            format!(
                r#"SELECT to_json({table}.*) FROM "{database}"."{table_schema}"."{table}" limit {limit} offset {page}"#,
                database = database.name,
                table = table.name,
                table_schema = table.schema.clone().unwrap_or_else(|| "public".to_string()),
                page = page,
                limit = RECORDS_LIMIT_PER_PAGE
            )
        };
        let json: Vec<(serde_json::Value,)> =
            sqlx::query_as(query.as_str()).fetch_all(&self.pool).await?;
        Ok(json.iter().map(|v| v.clone().0).collect())
    }
}

fn convert_column_value_to_string(row: &PgRow, column: &PgColumn) -> anyhow::Result<String> {
    let column_name = column.name();
    if let Ok(value) = row.try_get(column_name) {
        let value: Option<i16> = value;
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
        let value: Option<rust_decimal::Decimal> = value;
        return Ok(value.map_or("NULL".to_string(), |v| v.to_string()));
    }
    if let Ok(value) = row.try_get(column_name) {
        let value: Option<&[u8]> = value;
        return Ok(value.map_or("NULL".to_string(), |values| {
            format!(
                "\\x{}",
                values
                    .iter()
                    .map(|v| format!("{:02x}", v))
                    .collect::<String>()
            )
        }));
    }
    if let Ok(value) = row.try_get(column_name) {
        let value: Option<NaiveDate> = value;
        return Ok(value.map_or("NULL".to_string(), |v| v.to_string()));
    }
    if let Ok(value) = row.try_get(column_name) {
        let value: String = value;
        return Ok(value);
    }
    if let Ok(value) = row.try_get(column_name) {
        let value: Option<chrono::DateTime<chrono::Utc>> = value;
        return Ok(value.map_or("NULL".to_string(), |v| v.to_string()));
    }
    if let Ok(value) = row.try_get(column_name) {
        let value: Option<chrono::NaiveDateTime> = value;
        return Ok(value.map_or("NULL".to_string(), |v| v.to_string()));
    }
    if let Ok(value) = row.try_get::<Option<bool>, _>(column_name) {
        let value: Option<bool> = value;
        return Ok(value.map_or("NULL".to_string(), |v| v.to_string()));
    }
    if let Ok(value) = row.try_get(column_name) {
        let value: Option<Vec<String>> = value;
        return Ok(value.map_or("NULL".to_string(), |v| v.join(",").to_string()));
    }
    Err(anyhow::anyhow!(
        "column type not implemented: `{}` {}",
        column_name,
        column.type_info().clone().name()
    ))
}
