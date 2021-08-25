pub mod mysql;
pub mod postgres;

pub use mysql::MySqlPool;
pub use postgres::PostgresPool;

use async_trait::async_trait;
use database_tree::{Child, Database, Table};

pub const RECORDS_LIMIT_PER_PAGE: u8 = 200;

#[async_trait]
pub trait Pool {
    async fn get_databases(&self) -> anyhow::Result<Vec<Database>>;
    async fn get_tables(&self, database: String) -> anyhow::Result<Vec<Child>>;
    async fn get_records(
        &self,
        database: &Database,
        table: &Table,
        page: u16,
        filter: Option<String>,
    ) -> anyhow::Result<(Vec<String>, Vec<Vec<String>>)>;
    async fn get_columns(&self, database: &Database, table: &Table) -> anyhow::Result<Vec<Column>>;
    async fn get_constraints(
        &self,
        database: &Database,
        table: &Table,
    ) -> anyhow::Result<Vec<Constraint>>;
    async fn close(&self);
}

pub struct Constraint {
    name: String,
    column_name: String,
}

impl Constraint {
    pub fn headers() -> Vec<String> {
        vec!["name".to_string(), "column_name".to_string()]
    }

    pub fn columns(&self) -> Vec<String> {
        vec![self.name.to_string(), self.column_name.to_string()]
    }
}
