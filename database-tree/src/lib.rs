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
    pub children: Vec<Child>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Child {
    Table(Table),
    Schema(Schema),
}

impl From<Table> for Child {
    fn from(t: Table) -> Self {
        Child::Table(t)
    }
}

impl From<Schema> for Child {
    fn from(s: Schema) -> Self {
        Child::Schema(s)
    }
}

impl Database {
    pub fn new(database: String, children: Vec<Child>) -> Self {
        Self {
            name: database,
            children,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Schema {
    pub name: String,
    pub tables: Vec<Table>,
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
    pub schema: Option<String>,
}
