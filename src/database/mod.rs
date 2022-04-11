pub mod mysql;
pub mod postgres;
pub mod sqlite;

pub use mysql::MySqlPool;
pub use postgres::PostgresPool;
pub use sqlite::SqlitePool;

use async_trait::async_trait;
use database_tree::{Child, Database, Table};

pub const RECORDS_LIMIT_PER_PAGE: u8 = 200;

#[async_trait]
pub trait Pool: Send + Sync {
    async fn execute(&self, query: &String) -> anyhow::Result<ExecuteResult>;
    async fn get_databases(&self) -> anyhow::Result<Vec<Database>>;
    async fn get_tables(&self, database: String) -> anyhow::Result<Vec<Child>>;
    async fn get_records(
        &self,
        database: &Database,
        table: &Table,
        page: u16,
        filter: Option<String>,
        orders: Option<String>,
        header_icons: Option<Vec<String>>,
    ) -> anyhow::Result<(Vec<String>, Vec<Vec<String>>)>;
    async fn get_columns(
        &self,
        database: &Database,
        table: &Table,
    ) -> anyhow::Result<Vec<Box<dyn TableRow>>>;
    async fn get_constraints(
        &self,
        database: &Database,
        table: &Table,
    ) -> anyhow::Result<Vec<Box<dyn TableRow>>>;
    async fn get_foreign_keys(
        &self,
        database: &Database,
        table: &Table,
    ) -> anyhow::Result<Vec<Box<dyn TableRow>>>;
    async fn get_indexes(
        &self,
        database: &Database,
        table: &Table,
    ) -> anyhow::Result<Vec<Box<dyn TableRow>>>;
    async fn close(&self);
}

fn concat_headers(headers: Vec<String>, header_icons: Option<Vec<String>>) -> Vec<String> {
    if let Some(header_icons) = &header_icons {
        let mut new_headers = vec![String::new(); headers.len()];
        for (index, header) in headers.iter().enumerate() {
            new_headers[index] = format!("{} {}", header, header_icons[index])
                .trim()
                .to_string();
        }
        return new_headers;
    } else {
        return headers;
    }
}

pub enum ExecuteResult {
    Read {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
        database: Database,
        table: Table,
    },
    Write {
        updated_rows: u64,
    },
}

pub trait TableRow: std::marker::Send {
    fn fields(&self) -> Vec<String>;
    fn columns(&self) -> Vec<String>;
}

#[macro_export]
macro_rules! get_or_null {
    ($value:expr) => {
        $value.map_or("NULL".to_string(), |v| v.to_string())
    };
}

#[cfg(test)]
mod test {
    use super::concat_headers;
    #[test]
    fn test_concat_headers() {
        let headers = vec![
            "ID".to_string(),
            "NAME".to_string(),
            "TIMESTAMP".to_string(),
        ];
        let header_icons = vec!["".to_string(), "↑1".to_string(), "↓2".to_string()];
        let concat_headers: Vec<String> = concat_headers(headers, Some(header_icons));

        assert_eq!(
            concat_headers,
            vec![
                "ID".to_string(),
                "NAME ↑1".to_string(),
                "TIMESTAMP ↓2".to_string()
            ]
        )
    }
}
