use crate::Database;
use crate::{error::Result, treeitems_iter::TreeItemsIterator};
use crate::{item::DatabaseTreeItemKind, DatabaseTreeItem};
use std::{
    collections::{BTreeSet, HashMap},
    usize,
};

#[derive(Default)]
pub struct DatabaseTreeItems {
    pub tree_items: Vec<DatabaseTreeItem>,
}

impl DatabaseTreeItems {
    ///
    pub fn new(list: &[Database], collapsed: &BTreeSet<&String>) -> Result<Self> {
        Ok(Self {
            tree_items: Self::create_items(list, collapsed)?,
        })
    }

    fn create_items(
        list: &[Database],
        collapsed: &BTreeSet<&String>,
    ) -> Result<Vec<DatabaseTreeItem>> {
        let mut items = Vec::with_capacity(list.len());
        let mut items_added: HashMap<String, usize> = HashMap::with_capacity(list.len());

        for e in list {
            {
                Self::push_databases(e, &mut items, &mut items_added, collapsed)?;
            }
            for table in &e.tables {
                items.push(DatabaseTreeItem::new_table(e, table)?);
            }
        }

        Ok(items)
    }

    /// how many individual items (files/paths) are in the list
    pub fn len(&self) -> usize {
        self.tree_items.len()
    }

    /// iterates visible elements
    pub const fn iterate(&self, start: usize, max_amount: usize) -> TreeItemsIterator<'_> {
        TreeItemsIterator::new(self, start, max_amount)
    }

    fn push_databases<'a>(
        item_path: &'a Database,
        nodes: &mut Vec<DatabaseTreeItem>,
        // helps to only add new nodes for paths that were not added before
        // we also count the number of children a node has for later folding
        items_added: &mut HashMap<String, usize>,
        collapsed: &BTreeSet<&String>,
    ) -> Result<()> {
        let c = item_path.name.clone();
        if !items_added.contains_key(&c) {
            // add node and set count to have no children
            items_added.insert(c.clone(), 0);

            // increase the number of children in the parent node count
            *items_added.entry(item_path.name.clone()).or_insert(0) += 1;

            //TODO: make non alloc
            let path_string = c;
            let is_collapsed = collapsed.contains(&path_string);
            nodes.push(DatabaseTreeItem::new_database(item_path, is_collapsed)?);
        }

        // increase child count in parent node (the above ancenstor ignores the leaf component)
        *items_added.entry(item_path.name.clone()).or_insert(0) += 1;

        Ok(())
    }

    pub fn collapse(&mut self, index: usize, recursive: bool) {
        if self.tree_items[index].kind().is_database() {
            self.tree_items[index].collapse_database();

            let path = self.tree_items[index].info().full_path_str().to_string();

            for i in index + 1..self.tree_items.len() {
                let item = &mut self.tree_items[i];

                if recursive && item.kind().is_database() {
                    item.collapse_database();
                }

                if let Some(db) = item.kind().database_name() {
                    if db == path {
                        item.hide();
                    }
                } else {
                    return;
                }
            }
        }
    }

    pub fn expand(&mut self, index: usize, recursive: bool) {
        if self.tree_items[index].kind().is_database() {
            self.tree_items[index].expand_database();

            let full_path = self.tree_items[index].info().full_path_str().to_string();

            if recursive {
                for i in index + 1..self.tree_items.len() {
                    let item = &mut self.tree_items[i];

                    if let Some(db) = item.kind().database_name() {
                        if *db != full_path {
                            break;
                        }
                    }

                    if item.kind().is_database() && item.kind().is_database_collapsed() {
                        item.expand_database();
                    }
                }
            }

            self.update_visibility(&Some(full_path), index + 1, false);
        }
    }

    fn update_visibility(&mut self, prefix: &Option<String>, start_idx: usize, set_defaults: bool) {
        // if we are in any subpath that is collapsed we keep skipping over it
        let mut inner_collapsed: Option<String> = None;

        for i in start_idx..self.tree_items.len() {
            if let Some(ref collapsed_path) = inner_collapsed {
                if let Some(db) = self.tree_items[i].kind().database_name().clone() {
                    if db == *collapsed_path {
                        if set_defaults {
                            self.tree_items[i].info_mut().set_visible(false);
                        }
                        continue;
                    }
                }
                inner_collapsed = None;
            }

            let item_kind = self.tree_items[i].kind().clone();

            if matches!(item_kind, DatabaseTreeItemKind::Database{ collapsed, .. } if collapsed) {
                // we encountered an inner path that is still collapsed
                inner_collapsed = item_kind.database_name().clone();
            }

            if let Some(db) = item_kind.database_name() {
                if prefix.as_ref().map_or(true, |prefix| *prefix == *db) {
                    self.tree_items[i].info_mut().set_visible(true);
                }
            } else {
                // if we do not set defaults we can early out
                if set_defaults {
                    self.tree_items[i].info_mut().set_visible(false);
                } else {
                    return;
                }
            }
        }
    }
}
