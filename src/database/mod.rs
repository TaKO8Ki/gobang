pub mod mysql;

pub use mysql::MySqlPool;

use async_trait::async_trait;
use database_tree::{Database, Table};

pub const RECORDS_LIMIT_PER_PAGE: u8 = 200;

#[async_trait]
pub trait Pool {
    async fn get_databases(&self) -> anyhow::Result<Vec<Database>>;
    async fn get_tables(&self, database: String) -> anyhow::Result<Vec<Table>>;
    async fn get_records(
        &self,
        database: &str,
        table: &str,
        page: u16,
        filter: Option<String>,
    ) -> anyhow::Result<(Vec<String>, Vec<Vec<String>>)>;
    async fn get_columns(
        &self,
        database: &str,
        table: &str,
    ) -> anyhow::Result<(Vec<String>, Vec<Vec<String>>)>;
    async fn close(&self);
}
