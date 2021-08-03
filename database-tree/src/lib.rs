mod databasetree;
mod databasetreeitems;
mod error;
mod item;
mod tree_iter;
mod treeitems_iter;

pub use crate::{
    databasetree::DatabaseTree,
    databasetree::MoveSelection,
    item::{DatabaseTreeItem, TreeItemInfo},
};

#[derive(Clone, PartialEq, Debug)]
pub struct Database {
    pub name: String,
    pub tables: Vec<Table>,
}

impl Database {
    pub fn new(database: String, tables: Vec<Table>) -> Self {
        Self {
            name: database,
            tables,
        }
    }
}

#[derive(sqlx::FromRow, Debug, Clone, PartialEq)]
pub struct Table {
    #[sqlx(rename = "Name")]
    pub name: String,
    #[sqlx(rename = "Create_time")]
    pub create_time: Option<chrono::DateTime<chrono::Utc>>,
    #[sqlx(rename = "Update_time")]
    pub update_time: Option<chrono::DateTime<chrono::Utc>>,
    #[sqlx(rename = "Engine")]
    pub engine: Option<String>,
    #[sqlx(default)]
    pub table_schema: Option<String>,
}
