pub mod mysql;
pub mod postgres;
pub mod sqlite;

use async_trait::async_trait;
use database_tree::{Child, Database, Table};
pub use mysql::MySqlPool;
pub use postgres::PostgresPool;
pub use sqlite::SqlitePool;

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
    ) -> anyhow::Result<(Vec<String>, Vec<Vec<String>>)>;

    /// Get the total number of records in a table
    async fn get_total_records_count(
        &self,
        database: &Database,
        table: &Table,
        filter: Option<String>,
    ) -> anyhow::Result<usize>;

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
