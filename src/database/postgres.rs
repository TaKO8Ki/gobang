use super::{Pool, TableRow, RECORDS_LIMIT_PER_PAGE};
use async_trait::async_trait;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use database_tree::{Child, Database, Schema, Table};
use futures::TryStreamExt;
use itertools::Itertools;
use sqlx::postgres::{PgColumn, PgPool, PgPoolOptions, PgRow};
use sqlx::{Column as _, Row as _, TypeInfo as _};
use std::time::Duration;

pub struct PostgresPool {
    pool: PgPool,
}

impl PostgresPool {
    pub async fn new(database_url: &str) -> anyhow::Result<Self> {
        Ok(Self {
            pool: PgPoolOptions::new()
                .connect_timeout(Duration::from_millis(500))
                .connect(database_url)
                .await?,
        })
    }
}

pub struct Constraint {
    name: String,
    column_name: String,
}

impl TableRow for Constraint {
    fn fields(&self) -> Vec<String> {
        vec!["name".to_string(), "column_name".to_string()]
    }

    fn columns(&self) -> Vec<String> {
        vec![self.name.to_string(), self.column_name.to_string()]
    }
}

pub struct Column {
    name: Option<String>,
    r#type: Option<String>,
    null: Option<String>,
    default: Option<String>,
    comment: Option<String>,
}

impl TableRow for Column {
    fn fields(&self) -> Vec<String> {
        vec![
            "name".to_string(),
            "type".to_string(),
            "null".to_string(),
            "default".to_string(),
            "comment".to_string(),
        ]
    }

    fn columns(&self) -> Vec<String> {
        vec![
            self.name
                .as_ref()
                .map_or(String::new(), |name| name.to_string()),
            self.r#type
                .as_ref()
                .map_or(String::new(), |r#type| r#type.to_string()),
            self.null
                .as_ref()
                .map_or(String::new(), |null| null.to_string()),
            self.default
                .as_ref()
                .map_or(String::new(), |default| default.to_string()),
            self.comment
                .as_ref()
                .map_or(String::new(), |comment| comment.to_string()),
        ]
    }
}

pub struct ForeignKey {
    name: Option<String>,
    column_name: Option<String>,
    ref_table: Option<String>,
    ref_column: Option<String>,
}

impl TableRow for ForeignKey {
    fn fields(&self) -> Vec<String> {
        vec![
            "name".to_string(),
            "column_name".to_string(),
            "ref_table".to_string(),
            "ref_column".to_string(),
        ]
    }

    fn columns(&self) -> Vec<String> {
        vec![
            self.name
                .as_ref()
                .map_or(String::new(), |name| name.to_string()),
            self.column_name
                .as_ref()
                .map_or(String::new(), |r#type| r#type.to_string()),
            self.ref_table
                .as_ref()
                .map_or(String::new(), |r#type| r#type.to_string()),
            self.ref_column
                .as_ref()
                .map_or(String::new(), |r#type| r#type.to_string()),
        ]
    }
}

pub struct Index {
    name: Option<String>,
    column_name: Option<String>,
    r#type: Option<String>,
}

impl TableRow for Index {
    fn fields(&self) -> Vec<String> {
        vec![
            "name".to_string(),
            "column_name".to_string(),
            "type".to_string(),
        ]
    }

    fn columns(&self) -> Vec<String> {
        vec![
            self.name
                .as_ref()
                .map_or(String::new(), |name| name.to_string()),
            self.column_name
                .as_ref()
                .map_or(String::new(), |column_name| column_name.to_string()),
            self.r#type
                .as_ref()
                .map_or(String::new(), |r#type| r#type.to_string()),
        ]
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
                name: row.try_get("table_name")?,
                create_time: None,
                update_time: None,
                engine: None,
                schema: row.try_get("table_schema")?,
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
                                serde_json::Value::Number(v) => new_row.push(v.to_string()),
                                serde_json::Value::Bool(v) => new_row.push(v.to_string()),
                                others => {
                                    panic!(
                                        "column type not implemented: `{}` {}",
                                        column.name(),
                                        others
                                    )
                                }
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
    ) -> anyhow::Result<Vec<Box<dyn TableRow>>> {
        let table_schema = table
            .schema
            .as_ref()
            .map_or("public", |schema| schema.as_str());
        let mut rows = sqlx::query(
            "SELECT * FROM information_schema.columns WHERE table_catalog = $1 AND table_schema = $2 AND table_name = $3"
        )
        .bind(&database.name).bind(table_schema).bind(&table.name)
        .fetch(&self.pool);
        let mut columns: Vec<Box<dyn TableRow>> = vec![];
        while let Some(row) = rows.try_next().await? {
            columns.push(Box::new(Column {
                name: row.try_get("column_name")?,
                r#type: row.try_get("data_type")?,
                null: row.try_get("is_nullable")?,
                default: row.try_get("column_default")?,
                comment: None,
            }))
        }
        Ok(columns)
    }

    async fn get_constraints(
        &self,
        _database: &Database,
        table: &Table,
    ) -> anyhow::Result<Vec<Box<dyn TableRow>>> {
        let mut rows = sqlx::query(
            "
        SELECT
            tc.table_schema,
            tc.constraint_name,
            tc.table_name,
            kcu.column_name,
            ccu.table_schema AS foreign_table_schema,
            ccu.table_name AS foreign_table_name,
            ccu.column_name AS foreign_column_name
        FROM
            information_schema.table_constraints AS tc
            JOIN information_schema.key_column_usage AS kcu ON tc.constraint_name = kcu.constraint_name
            AND tc.table_schema = kcu.table_schema
            JOIN information_schema.constraint_column_usage AS ccu ON ccu.constraint_name = tc.constraint_name
            AND ccu.table_schema = tc.table_schema
        WHERE
            NOT tc.constraint_type = 'FOREIGN KEY'
            AND tc.table_name = $1
        ",
        )
        .bind(&table.name)
        .fetch(&self.pool);
        let mut constraints: Vec<Box<dyn TableRow>> = vec![];
        while let Some(row) = rows.try_next().await? {
            constraints.push(Box::new(Constraint {
                name: row.try_get("constraint_name")?,
                column_name: row.try_get("column_name")?,
            }))
        }
        Ok(constraints)
    }

    async fn get_foreign_keys(
        &self,
        _database: &Database,
        table: &Table,
    ) -> anyhow::Result<Vec<Box<dyn TableRow>>> {
        let mut rows = sqlx::query(
            "
        SELECT
            tc.table_schema,
            tc.constraint_name,
            tc.table_name,
            kcu.column_name,
            ccu.table_schema AS foreign_table_schema,
            ccu.table_name AS foreign_table_name,
            ccu.column_name AS foreign_column_name
        FROM
            information_schema.table_constraints AS tc
            JOIN information_schema.key_column_usage AS kcu ON tc.constraint_name = kcu.constraint_name
            AND tc.table_schema = kcu.table_schema
            JOIN information_schema.constraint_column_usage AS ccu ON ccu.constraint_name = tc.constraint_name
            AND ccu.table_schema = tc.table_schema
        WHERE
            tc.constraint_type = 'FOREIGN KEY'
            AND tc.table_name = $1
        ",
        )
        .bind(&table.name)
        .fetch(&self.pool);
        let mut constraints: Vec<Box<dyn TableRow>> = vec![];
        while let Some(row) = rows.try_next().await? {
            constraints.push(Box::new(ForeignKey {
                name: row.try_get("constraint_name")?,
                column_name: row.try_get("column_name")?,
                ref_table: row.try_get("foreign_table_name")?,
                ref_column: row.try_get("foreign_column_name")?,
            }))
        }
        Ok(constraints)
    }

    async fn get_indexes(
        &self,
        _database: &Database,
        table: &Table,
    ) -> anyhow::Result<Vec<Box<dyn TableRow>>> {
        let mut rows = sqlx::query(
            "
        SELECT
            t.relname AS table_name,
            i.relname AS index_name,
            a.attname AS column_name,
            am.amname AS type
        FROM
            pg_class t,
            pg_class i,
            pg_index ix,
            pg_attribute a,
            pg_am am
        WHERE
            t.oid = ix.indrelid
            and i.oid = ix.indexrelid
            and a.attrelid = t.oid
            and a.attnum = ANY(ix.indkey)
            and t.relkind = 'r'
            and am.oid = i.relam
            and t.relname = $1
        ORDER BY
            t.relname,
            i.relname
        ",
        )
        .bind(&table.name)
        .fetch(&self.pool);
        let mut foreign_keys: Vec<Box<dyn TableRow>> = vec![];
        while let Some(row) = rows.try_next().await? {
            foreign_keys.push(Box::new(Index {
                name: row.try_get("index_name")?,
                column_name: row.try_get("column_name")?,
                r#type: row.try_get("type")?,
            }))
        }
        Ok(foreign_keys)
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
        let value: Option<chrono::DateTime<chrono::Local>> = value;
        return Ok(value.map_or("NULL".to_string(), |v| v.to_string()));
    }
    if let Ok(value) = row.try_get(column_name) {
        let value: Option<NaiveDateTime> = value;
        return Ok(value.map_or("NULL".to_string(), |v| v.to_string()));
    }
    if let Ok(value) = row.try_get(column_name) {
        let value: Option<NaiveDate> = value;
        return Ok(value.map_or("NULL".to_string(), |v| v.to_string()));
    }
    if let Ok(value) = row.try_get(column_name) {
        let value: Option<NaiveTime> = value;
        return Ok(value.map_or("NULL".to_string(), |v| v.to_string()));
    }
    if let Ok(value) = row.try_get(column_name) {
        let value: Option<serde_json::Value> = value;
        return Ok(value.map_or("NULL".to_string(), |v| v.to_string()));
    }
    if let Ok(value) = row.try_get::<Option<bool>, _>(column_name) {
        let value: Option<bool> = value;
        return Ok(value.map_or("NULL".to_string(), |v| v.to_string()));
    }
    if let Ok(value) = row.try_get(column_name) {
        let value: Option<Vec<String>> = value;
        return Ok(value.map_or("NULL".to_string(), |v| v.join(",")));
    }
    Err(anyhow::anyhow!(
        "column type not implemented: `{}` {}",
        column_name,
        column.type_info().clone().name()
    ))
}
