use crate::{error::Result, treeitems_iter::TreeItemsIterator};
use crate::{item::DatabaseTreeItemKind, DatabaseTreeItem};
use crate::{Child, Database};
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

    pub fn filter(&self, filter_text: String) -> Self {
        Self {
            tree_items: self
                .tree_items
                .iter()
                .filter(|item| item.is_database() || item.is_match(&filter_text))
                .map(|item| {
                    if item.is_database() {
                        let mut item = item.clone();
                        item.set_collapsed(false);
                        item
                    } else {
                        let mut item = item.clone();
                        item.show();
                        item
                    }
                })
                .collect::<Vec<DatabaseTreeItem>>(),
        }
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
            for child in &e.children {
                match child {
                    Child::Table(table) => items.push(DatabaseTreeItem::new_table(e, table)),
                    Child::Schema(schema) => {
                        items.push(DatabaseTreeItem::new_schema(e, schema, true));
                        for table in &schema.tables {
                            items.push(DatabaseTreeItem::new_table(e, table))
                        }
                    }
                }
            }
        }

        Ok(items)
    }

    /// how many individual items are in the list
    pub fn len(&self) -> usize {
        self.tree_items.len()
    }

    /// iterates visible elements
    pub const fn iterate(&self, start: usize, max_amount: usize) -> TreeItemsIterator<'_> {
        TreeItemsIterator::new(self, start, max_amount)
    }

    fn push_databases<'a>(
        database: &'a Database,
        nodes: &mut Vec<DatabaseTreeItem>,
        items_added: &mut HashMap<String, usize>,
        collapsed: &BTreeSet<&String>,
    ) -> Result<()> {
        let c = database.name.clone();
        if !items_added.contains_key(&c) {
            // add node and set count to have no children
            items_added.insert(c.clone(), 0);

            // increase the number of children in the parent node count
            *items_added.entry(database.name.clone()).or_insert(0) += 1;

            let is_collapsed = collapsed.contains(&c);
            nodes.push(DatabaseTreeItem::new_database(database, is_collapsed));
        }

        // increase child count in parent node (the above ancenstor ignores the leaf component)
        *items_added.entry(database.name.clone()).or_insert(0) += 1;

        Ok(())
    }

    pub fn collapse(&mut self, index: usize, recursive: bool) {
        if self.tree_items[index].kind().is_database() {
            self.tree_items[index].collapse_database();

            let name = self.tree_items[index].kind().name();

            for i in index + 1..self.tree_items.len() {
                let item = &mut self.tree_items[i];

                if recursive && item.kind().is_database() {
                    item.collapse_database();
                }

                if let Some(db) = item.kind().database_name() {
                    if db == name {
                        item.hide();
                    }
                } else {
                    return;
                }
            }
        }

        if self.tree_items[index].kind().is_schema() {
            self.tree_items[index].collapse_schema();

            let name = self.tree_items[index].kind().name();

            for i in index + 1..self.tree_items.len() {
                let item = &mut self.tree_items[i];

                if recursive && item.kind().is_schema() {
                    item.collapse_schema();
                }

                if let Some(schema) = item.kind().schema_name() {
                    if schema == name {
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

            let tree_item = self.tree_items[index].clone();
            let name = self.tree_items[index].kind().name();
            let kind = tree_item.kind();

            if recursive {
                for i in index + 1..self.tree_items.len() {
                    let item = &mut self.tree_items[i];

                    if let Some(db) = item.kind().database_name() {
                        if *db != name {
                            break;
                        }
                    }

                    if item.kind().is_database() && item.kind().is_database_collapsed() {
                        item.expand_database();
                    }
                }
            }

            self.update_visibility(kind, index + 1);
        }

        if self.tree_items[index].kind().is_schema() {
            self.tree_items[index].expand_schema();

            let tree_item = self.tree_items[index].clone();
            let name = self.tree_items[index].kind().name();
            let kind = tree_item.kind();

            if recursive {
                for i in index + 1..self.tree_items.len() {
                    let item = &mut self.tree_items[i];

                    if let Some(schema) = item.kind().schema_name() {
                        if *schema != name {
                            break;
                        }
                    }

                    if item.kind().is_schema() && item.kind().is_schema_collapsed() {
                        item.expand_schema();
                    }
                }
            }

            self.update_visibility(kind, index + 1);
        }
    }

    fn update_visibility(&mut self, prefix: &DatabaseTreeItemKind, start_idx: usize) {
        let mut inner_collapsed: Option<DatabaseTreeItemKind> = None;

        for i in start_idx..self.tree_items.len() {
            if let Some(ref collapsed_item) = inner_collapsed {
                match collapsed_item {
                    DatabaseTreeItemKind::Database { name, .. } => {
                        if let DatabaseTreeItemKind::Schema { database, .. } =
                            self.tree_items[i].kind().clone()
                        {
                            if database.name == *name {
                                continue;
                            }
                        }
                        if let DatabaseTreeItemKind::Table { database, .. } =
                            self.tree_items[i].kind().clone()
                        {
                            if database.name == *name {
                                continue;
                            }
                        }
                    }
                    DatabaseTreeItemKind::Schema { schema, .. } => {
                        if let DatabaseTreeItemKind::Table { table, .. } =
                            self.tree_items[i].kind().clone()
                        {
                            if matches!(table.schema, Some(table_schema) if schema.name == table_schema)
                            {
                                continue;
                            }
                        }
                    }
                    _ => (),
                }
                inner_collapsed = None;
            }

            let item_kind = self.tree_items[i].kind().clone();

            if matches!(item_kind, DatabaseTreeItemKind::Database{ collapsed, .. } if collapsed) {
                inner_collapsed = Some(item_kind.clone());
            } else if matches!(item_kind, DatabaseTreeItemKind::Schema{ collapsed, .. } if collapsed)
            {
                inner_collapsed = Some(item_kind.clone());
            }

            match prefix {
                DatabaseTreeItemKind::Database { name, .. } => {
                    if let DatabaseTreeItemKind::Schema { database, .. } = item_kind.clone() {
                        if *name == database.name {
                            self.tree_items[i].info_mut().set_visible(true);
                        }
                    }

                    if let DatabaseTreeItemKind::Table { database, .. } = item_kind {
                        if *name == database.name {
                            self.tree_items[i].info_mut().set_visible(true);
                        }
                    }
                }
                DatabaseTreeItemKind::Schema { schema, .. } => {
                    if let DatabaseTreeItemKind::Table { table, .. } = item_kind {
                        if matches!(table.schema, Some(table_schema) if schema.name == table_schema)
                        {
                            self.tree_items[i].info_mut().set_visible(true);
                        }
                    }
                }
                _ => (),
            }
        }
    }
}
