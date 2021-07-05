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

#[derive(Clone)]
pub struct Database {
    pub name: String,
    pub tables: Vec<String>,
}
