use async_trait::async_trait;
use chrono::NaiveDate;
use database_tree::{Database, Table};
use futures::TryStreamExt;
use sqlx::mysql::{MySqlColumn, MySqlPool as MPool, MySqlRow};
use sqlx::{Column as _, Row, TypeInfo};

pub const RECORDS_LIMIT_PER_PAGE: u8 = 200;

#[async_trait]
pub trait Pool {
    async fn get_databases(&self) -> anyhow::Result<Vec<Database>>;
    async fn get_tables(&self, database: String) -> anyhow::Result<Vec<Table>>;
    async fn get_records(
        &self,
        database: &String,
        table: &String,
        page: u16,
        filter: Option<String>,
    ) -> anyhow::Result<(Vec<String>, Vec<Vec<String>>)>;
    async fn get_columns(
        &self,
        database: &String,
        table: &String,
    ) -> anyhow::Result<(Vec<String>, Vec<Vec<String>>)>;
    async fn close(&self);
}

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
                get_tables(db.clone(), &self.pool).await?,
            ))
        }
        Ok(list)
    }

    async fn get_tables(&self, database: String) -> anyhow::Result<Vec<Table>> {
        let tables =
            sqlx::query_as::<_, Table>(format!("SHOW TABLE STATUS FROM `{}`", database).as_str())
                .fetch_all(&self.pool)
                .await?;
        Ok(tables)
    }

    async fn get_records(
        &self,
        database: &String,
        table: &String,
        page: u16,
        filter: Option<String>,
    ) -> anyhow::Result<(Vec<String>, Vec<Vec<String>>)> {
        let query = if let Some(filter) = filter {
            format!(
                "SELECT * FROM `{database}`.`{table}` WHERE {filter} LIMIT {page}, {limit}",
                database = database,
                table = table,
                filter = filter,
                page = page,
                limit = RECORDS_LIMIT_PER_PAGE
            )
        } else {
            format!(
                "SELECT * FROM `{}`.`{}` limit {page}, {limit}",
                database,
                table,
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
            records.push(
                row.columns()
                    .iter()
                    .map(|col| convert_column_value_to_string(&row, col))
                    .collect::<Vec<String>>(),
            )
        }
        Ok((headers, records))
    }

    async fn get_columns(
        &self,
        database: &String,
        table: &String,
    ) -> anyhow::Result<(Vec<String>, Vec<Vec<String>>)> {
        let query = format!("SHOW FULL COLUMNS FROM `{}`.`{}`", database, table);
        let mut rows = sqlx::query(query.as_str()).fetch(&self.pool);
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

    async fn close(&self) {
        self.pool.close().await;
    }
}

pub async fn get_tables(database: String, pool: &MPool) -> anyhow::Result<Vec<Table>> {
    let tables =
        sqlx::query_as::<_, Table>(format!("SHOW TABLE STATUS FROM `{}`", database).as_str())
            .fetch_all(pool)
            .await?;
    Ok(tables)
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
