use crate::{
    databasetreeitems::DatabaseTreeItems, error::Result, item::DatabaseTreeItemKind,
    tree_iter::TreeIterator,
};
use crate::{Database, Table};
use std::{collections::BTreeSet, usize};

///
#[derive(Copy, Clone, Debug)]
pub enum MoveSelection {
    Up,
    Down,
    MultipleUp,
    MultipleDown,
    Left,
    Right,
    Top,
    End,
    Enter,
}

#[derive(Debug, Clone, Copy)]
pub struct VisualSelection {
    pub count: usize,
    pub index: usize,
}

/// wraps `DatabaseTreeItems` as a datastore and adds selection functionality
#[derive(Default)]
pub struct DatabaseTree {
    items: DatabaseTreeItems,
    pub selection: Option<usize>,
    visual_selection: Option<VisualSelection>,
}

impl DatabaseTree {
    pub fn new(list: &[crate::Database], collapsed: &BTreeSet<&String>) -> Result<Self> {
        let mut new_self = Self {
            items: DatabaseTreeItems::new(list, collapsed)?,
            selection: if list.is_empty() { None } else { Some(0) },
            visual_selection: None,
        };
        new_self.visual_selection = new_self.calc_visual_selection();

        Ok(new_self)
    }

    pub fn filter(&self, filter_text: String) -> Self {
        let mut new_self = Self {
            items: self.items.filter(filter_text),
            selection: Some(0),
            visual_selection: None,
        };
        new_self.visual_selection = new_self.calc_visual_selection();
        new_self
    }

    pub fn collapse_but_root(&mut self) {
        self.items.collapse(0, true);
        self.items.expand(0, false);
    }

    /// iterates visible elements starting from `start_index_visual`
    pub fn iterate(&self, start_index_visual: usize, max_amount: usize) -> TreeIterator<'_> {
        let start = self
            .visual_index_to_absolute(start_index_visual)
            .unwrap_or_default();
        TreeIterator::new(self.items.iterate(start, max_amount), self.selection)
    }

    pub const fn visual_selection(&self) -> Option<&VisualSelection> {
        self.visual_selection.as_ref()
    }

    pub fn selected_item(&self) -> Option<&crate::DatabaseTreeItem> {
        self.selection
            .and_then(|index| self.items.tree_items.get(index))
    }

    pub fn selected_table(&self) -> Option<(Database, Table)> {
        self.selection.and_then(|index| {
            let item = &self.items.tree_items[index];
            match item.kind() {
                DatabaseTreeItemKind::Database { .. } => None,
                DatabaseTreeItemKind::Table { table, database } => {
                    Some((database.clone(), table.clone()))
                }
                DatabaseTreeItemKind::Schema { .. } => None,
            }
        })
    }

    pub fn collapse_recursive(&mut self) {
        if let Some(selection) = self.selection {
            self.items.collapse(selection, true);
        }
    }

    pub fn expand_recursive(&mut self) {
        if let Some(selection) = self.selection {
            self.items.expand(selection, true);
        }
    }

    pub fn move_selection(&mut self, dir: MoveSelection) -> bool {
        self.selection.map_or(false, |selection| {
            let new_index = match dir {
                MoveSelection::Up => self.selection_up(selection, 1),
                MoveSelection::Down => self.selection_down(selection, 1),
                MoveSelection::MultipleUp => self.selection_up(selection, 10),
                MoveSelection::MultipleDown => self.selection_down(selection, 10),
                MoveSelection::Left => self.selection_left(selection),
                MoveSelection::Right => self.selection_right(selection),
                MoveSelection::Top => Self::selection_start(selection),
                MoveSelection::End => self.selection_end(selection),
                MoveSelection::Enter => self.expand(selection),
            };

            let changed_index = new_index.map(|i| i != selection).unwrap_or_default();

            if changed_index {
                self.selection = new_index;
                self.visual_selection = self.calc_visual_selection();
            }

            changed_index || new_index.is_some()
        })
    }

    fn visual_index_to_absolute(&self, visual_index: usize) -> Option<usize> {
        self.items
            .iterate(0, self.items.len())
            .enumerate()
            .find_map(
                |(i, (abs, _))| {
                    if i == visual_index {
                        Some(abs)
                    } else {
                        None
                    }
                },
            )
    }

    fn calc_visual_selection(&self) -> Option<VisualSelection> {
        self.selection.map(|selection_absolute| {
            let mut count = 0;
            let mut visual_index = 0;
            for (index, _item) in self.items.iterate(0, self.items.len()) {
                if selection_absolute == index {
                    visual_index = count;
                }

                count += 1;
            }

            VisualSelection {
                index: visual_index,
                count,
            }
        })
    }

    const fn selection_start(current_index: usize) -> Option<usize> {
        if current_index == 0 {
            None
        } else {
            Some(0)
        }
    }

    fn selection_end(&self, current_index: usize) -> Option<usize> {
        let items_max = self.items.len().saturating_sub(1);

        let mut new_index = items_max;

        loop {
            if self.is_visible_index(new_index) {
                break;
            }

            if new_index == 0 {
                break;
            }

            new_index = new_index.saturating_sub(1);
            new_index = std::cmp::min(new_index, items_max);
        }

        if new_index == current_index {
            None
        } else {
            Some(new_index)
        }
    }

    fn selection_up(&self, current_index: usize, lines: usize) -> Option<usize> {
        let mut index = current_index;

        'a: for _ in 0..lines {
            loop {
                if index == 0 {
                    break 'a;
                }

                index = index.saturating_sub(1);

                if self.is_visible_index(index) {
                    break;
                }
            }
        }

        if index == current_index {
            None
        } else {
            Some(index)
        }
    }

    fn selection_down(&self, current_index: usize, lines: usize) -> Option<usize> {
        let mut index = current_index;
        let last_visible_item_index = self
            .items
            .tree_items
            .iter()
            .rposition(|x| x.info().is_visible())?;

        'a: for _ in 0..lines {
            loop {
                if index >= last_visible_item_index {
                    break 'a;
                }

                index = index.saturating_add(1);

                if self.is_visible_index(index) {
                    break;
                }
            }
        }

        if index == current_index {
            None
        } else {
            Some(index)
        }
    }

    fn selection_updown(&self, current_index: usize, up: bool) -> Option<usize> {
        let mut index = current_index;

        loop {
            index = {
                let new_index = if up {
                    index.saturating_sub(1)
                } else {
                    index.saturating_add(1)
                };

                if new_index == index {
                    break;
                }

                if new_index >= self.items.len() {
                    break;
                }

                new_index
            };

            if self.is_visible_index(index) {
                break;
            }
        }

        if index == current_index {
            None
        } else {
            Some(index)
        }
    }

    fn select_parent(&mut self, current_index: usize) -> Option<usize> {
        let indent = self.items.tree_items.get(current_index)?.info().indent();

        let mut index = current_index;

        while let Some(selection) = self.selection_updown(index, true) {
            index = selection;

            if self.items.tree_items[index].info().indent() < indent {
                break;
            }
        }

        if index == current_index {
            None
        } else {
            Some(index)
        }
    }

    fn selection_left(&mut self, current_index: usize) -> Option<usize> {
        let item = &mut self.items.tree_items.get(current_index)?;

        if item.kind().is_database() && !item.kind().is_database_collapsed() {
            self.items.collapse(current_index, false);
            return Some(current_index);
        }

        if item.kind().is_schema() && !item.kind().is_schema_collapsed() {
            self.items.collapse(current_index, false);
            return Some(current_index);
        }

        self.select_parent(current_index)
    }

    fn expand(&mut self, current_selection: usize) -> Option<usize> {
        let item = &mut self.items.tree_items.get(current_selection)?;

        if item.kind().is_database() && item.kind().is_database_collapsed() {
            self.items.expand(current_selection, false);
            return Some(current_selection);
        }

        if item.kind().is_schema() && item.kind().is_schema_collapsed() {
            self.items.expand(current_selection, false);
            return Some(current_selection);
        }

        None
    }

    fn selection_right(&mut self, current_selection: usize) -> Option<usize> {
        let item = &mut self.items.tree_items.get(current_selection)?;

        if item.kind().is_database() {
            if item.kind().is_database_collapsed() {
                self.items.expand(current_selection, false);
                return Some(current_selection);
            }
            return self.selection_updown(current_selection, false);
        }

        if item.kind().is_schema() {
            if item.kind().is_schema_collapsed() {
                self.items.expand(current_selection, false);
                return Some(current_selection);
            }
            return self.selection_updown(current_selection, false);
        }

        None
    }

    fn is_visible_index(&self, index: usize) -> bool {
        self.items
            .tree_items
            .get(index)
            .map(|item| item.info().is_visible())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod test {
    use crate::{Database, DatabaseTree, MoveSelection, Schema, Table};
    use std::collections::BTreeSet;

    impl Table {
        fn new(name: String) -> Self {
            Table {
                name,
                create_time: None,
                update_time: None,
                engine: None,
                schema: None,
            }
        }

        fn new_with_schema(name: String, schema: String) -> Self {
            Table {
                name,
                create_time: None,
                update_time: None,
                engine: None,
                schema: Some(schema),
            }
        }
    }

    #[test]
    fn test_selection() {
        let items = vec![Database::new(
            "a".to_string(),
            vec![Table::new("b".to_string()).into()],
        )];

        // a
        //   b

        let mut tree = DatabaseTree::new(&items, &BTreeSet::new()).unwrap();

        assert!(tree.move_selection(MoveSelection::Right));
        assert_eq!(tree.selection, Some(0));
        assert!(tree.move_selection(MoveSelection::Down));
        assert_eq!(tree.selection, Some(1));

        let items = vec![Database::new(
            "a".to_string(),
            vec![Schema {
                name: "b".to_string(),
                tables: vec![Table::new("c".to_string()).into()],
            }
            .into()],
        )];

        // a
        //   b
        //     c

        let mut tree = DatabaseTree::new(&items, &BTreeSet::new()).unwrap();

        assert!(tree.move_selection(MoveSelection::Right));
        assert_eq!(tree.selection, Some(0));
        assert!(tree.move_selection(MoveSelection::Down));
        assert_eq!(tree.selection, Some(1));
        assert!(tree.move_selection(MoveSelection::Right));
        assert_eq!(tree.selection, Some(1));
        assert!(tree.move_selection(MoveSelection::Down));
        assert_eq!(tree.selection, Some(2));
    }

    #[test]
    fn test_expand() {
        let items = vec![Database::new(
            "a".to_string(),
            vec![Table::new("b".to_string()).into()],
        )];

        // a
        //   b

        let mut tree = DatabaseTree::new(&items, &BTreeSet::new()).unwrap();

        assert!(tree.move_selection(MoveSelection::Enter));
        assert!(!tree.items.tree_items[0].kind().is_database_collapsed());
        assert_eq!(tree.selection, Some(0));

        assert!(tree.move_selection(MoveSelection::Down));
        assert_eq!(tree.selection, Some(1));

        assert!(!tree.move_selection(MoveSelection::Enter));
        assert_eq!(tree.selection, Some(1));

        let items = vec![Database::new(
            "a".to_string(),
            vec![Schema {
                name: "b".to_string(),
                tables: vec![Table::new("c".to_string()).into()],
            }
            .into()],
        )];

        // a
        //   b
        //     c

        let mut tree = DatabaseTree::new(&items, &BTreeSet::new()).unwrap();

        assert!(tree.move_selection(MoveSelection::Enter));
        assert!(!tree.items.tree_items[0].kind().is_database_collapsed());
        assert!(tree.items.tree_items[1].kind().is_schema_collapsed());
        assert_eq!(tree.selection, Some(0));

        assert!(tree.move_selection(MoveSelection::Down));
        assert_eq!(tree.selection, Some(1));

        assert!(tree.move_selection(MoveSelection::Enter));
        assert!(!tree.items.tree_items[1].kind().is_database_collapsed());
        assert_eq!(tree.selection, Some(1));

        assert!(tree.move_selection(MoveSelection::Down));
        assert_eq!(tree.selection, Some(2));

        assert!(!tree.move_selection(MoveSelection::Enter));
        assert_eq!(tree.selection, Some(2));
    }

    #[test]
    fn test_selection_multiple_up_down() {
        let items = vec![Database::new(
            "a".to_string(),
            vec!["b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l"]
                .iter()
                .map(|x| Table::new(x.to_string()).into())
                .collect(),
        )];

        // a
        //   b
        //   ...
        //   j

        let mut tree = DatabaseTree::new(&items, &BTreeSet::new()).unwrap();

        assert!(tree.move_selection(MoveSelection::Right));
        assert_eq!(tree.selection, Some(0));
        assert!(tree.move_selection(MoveSelection::MultipleDown));
        assert_eq!(tree.selection, Some(10));

        tree.selection = Some(11);
        assert!(tree.move_selection(MoveSelection::MultipleUp));
        assert_eq!(tree.selection, Some(1));

        let items = vec![Database::new(
            "a".to_string(),
            vec![Schema {
                name: "b".to_string(),
                tables: vec!["c", "d", "e", "f", "g", "h", "i", "j", "k", "l"]
                    .iter()
                    .map(|x| Table::new(x.to_string()).into())
                    .collect(),
            }
            .into()],
        )];

        // a
        //   b
        //     c
        //     ...
        //     l

        let mut tree = DatabaseTree::new(&items, &BTreeSet::new()).unwrap();

        assert!(tree.move_selection(MoveSelection::Right));
        assert_eq!(tree.selection, Some(0));
        assert!(tree.move_selection(MoveSelection::MultipleDown));
        assert_eq!(tree.selection, Some(10));

        tree.selection = Some(11);
        assert!(tree.move_selection(MoveSelection::MultipleUp));
        assert_eq!(tree.selection, Some(1));
    }

    #[test]
    fn test_selection_skips_collapsed() {
        let items = vec![
            Database::new(
                "a".to_string(),
                vec![
                    Table::new("b".to_string()).into(),
                    Table::new("c".to_string()).into(),
                ],
            ),
            Database::new("d".to_string(), vec![Table::new("e".to_string()).into()]),
        ];

        // a
        //   b
        //   c
        // d
        //   e

        let mut tree = DatabaseTree::new(&items, &BTreeSet::new()).unwrap();
        tree.selection = Some(1);

        assert!(tree.move_selection(MoveSelection::Down));
        assert_eq!(tree.selection, Some(3));

        let items = vec![
            Database::new(
                "a".to_string(),
                vec![Schema {
                    name: "b".to_string(),
                    tables: vec![Table::new("c".to_string()).into()],
                }
                .into()],
            ),
            Database::new(
                "d".to_string(),
                vec![Schema {
                    name: "e".to_string(),
                    tables: vec![Table::new("f".to_string()).into()],
                }
                .into()],
            ),
        ];

        // a
        //   b
        //     c
        // d
        //   e
        //     f

        let mut tree = DatabaseTree::new(&items, &BTreeSet::new()).unwrap();
        tree.selection = Some(1);

        assert!(tree.move_selection(MoveSelection::Down));
        assert_eq!(tree.selection, Some(3));
    }

    #[test]
    fn test_selection_left_collapse() {
        let items = vec![Database::new(
            "a".to_string(),
            vec![
                Table::new("b".to_string()).into(),
                Table::new("c".to_string()).into(),
            ],
        )];

        // a
        //   b
        //   c

        let mut tree = DatabaseTree::new(&items, &BTreeSet::new()).unwrap();
        tree.selection = Some(0);
        tree.items.expand(0, false);

        assert!(tree.move_selection(MoveSelection::Left));
        assert_eq!(tree.selection, Some(0));
        assert!(tree.items.tree_items[0].kind().is_database_collapsed());
        assert!(!tree.items.tree_items[1].info().is_visible());
        assert!(!tree.items.tree_items[2].info().is_visible());

        let items = vec![
            Database::new(
                "a".to_string(),
                vec![Schema {
                    name: "b".to_string(),
                    tables: vec![Table::new("c".to_string()).into()],
                }
                .into()],
            ),
            Database::new(
                "d".to_string(),
                vec![Schema {
                    name: "e".to_string(),
                    tables: vec![Table::new("f".to_string()).into()],
                }
                .into()],
            ),
        ];

        // a
        //   b
        //     c
        // d
        //   e
        //     f

        let mut tree = DatabaseTree::new(&items, &BTreeSet::new()).unwrap();
        tree.selection = Some(0);
        tree.items.expand(0, false);

        assert!(tree.move_selection(MoveSelection::Left));
        assert_eq!(tree.selection, Some(0));
        assert!(tree.items.tree_items[0].kind().is_database_collapsed());
        assert!(!tree.items.tree_items[1].info().is_visible());
        assert!(!tree.items.tree_items[2].info().is_visible());
    }

    #[test]
    fn test_selection_left_parent() {
        let items = vec![Database::new(
            "a".to_string(),
            vec![
                Table::new("b".to_string()).into(),
                Table::new("c".to_string()).into(),
            ],
        )];

        // a
        //   b
        //   c

        let mut tree = DatabaseTree::new(&items, &BTreeSet::new()).unwrap();
        tree.selection = Some(2);
        tree.items.expand(0, false);

        assert!(tree.move_selection(MoveSelection::Left));
        assert_eq!(tree.selection, Some(0));

        let items = vec![Database::new(
            "a".to_string(),
            vec![Schema {
                name: "b".to_string(),
                tables: vec![Table::new_with_schema("c".to_string(), "a".to_string()).into()],
            }
            .into()],
        )];

        // a
        //   b
        //     c

        let mut tree = DatabaseTree::new(&items, &BTreeSet::new()).unwrap();

        tree.selection = Some(2);
        tree.items.expand(0, false);
        tree.items.expand(1, false);
        assert!(tree.move_selection(MoveSelection::Left));
        assert_eq!(tree.selection, Some(1));

        assert!(tree.move_selection(MoveSelection::Left));
        assert_eq!(tree.selection, Some(1));

        assert!(tree.move_selection(MoveSelection::Left));
        assert_eq!(tree.selection, Some(0));
    }

    #[test]
    fn test_selection_right_expand() {
        let items = vec![Database::new(
            "a".to_string(),
            vec![
                Table::new("b".to_string()).into(),
                Table::new("c".to_string()).into(),
            ],
        )];

        // a
        //   b
        //   c

        let mut tree = DatabaseTree::new(&items, &BTreeSet::new()).unwrap();
        tree.selection = Some(0);

        assert!(tree.move_selection(MoveSelection::Right));
        assert_eq!(tree.selection, Some(0));
        assert!(!tree.items.tree_items[0].kind().is_database_collapsed());

        assert!(tree.move_selection(MoveSelection::Right));
        assert_eq!(tree.selection, Some(1));

        let items = vec![Database::new(
            "a".to_string(),
            vec![Schema {
                name: "b".to_string(),
                tables: vec![Table::new_with_schema("c".to_string(), "a".to_string()).into()],
            }
            .into()],
        )];

        // a
        //   b
        //     c

        let mut tree = DatabaseTree::new(&items, &BTreeSet::new()).unwrap();
        tree.selection = Some(0);

        assert!(tree.move_selection(MoveSelection::Right));
        assert_eq!(tree.selection, Some(0));
        assert!(!tree.items.tree_items[0].kind().is_database_collapsed());

        assert!(tree.move_selection(MoveSelection::Right));
        assert_eq!(tree.selection, Some(1));

        assert!(tree.move_selection(MoveSelection::Right));
        assert_eq!(tree.selection, Some(1));
        assert!(!tree.items.tree_items[0].kind().is_schema_collapsed());
    }

    #[test]
    fn test_visible_selection() {
        let items = vec![
            Database::new(
                "a".to_string(),
                vec![
                    Table::new("b".to_string()).into(),
                    Table::new("c".to_string()).into(),
                ],
            ),
            Database::new("d".to_string(), vec![Table::new("e".to_string()).into()]),
        ];

        // a
        //   b
        //   c
        // d
        //   e

        let mut tree = DatabaseTree::new(&items, &BTreeSet::new()).unwrap();
        tree.items.expand(0, false);
        tree.items.expand(3, false);

        tree.selection = Some(0);
        assert!(tree.move_selection(MoveSelection::Left));
        assert!(tree.move_selection(MoveSelection::Down));
        assert_eq!(tree.selection, Some(3));
        let s = tree.visual_selection().unwrap();

        assert_eq!(s.count, 3);
        assert_eq!(s.index, 1);

        let items = vec![
            Database::new(
                "a".to_string(),
                vec![Schema {
                    name: "b".to_string(),
                    tables: vec![Table::new_with_schema("c".to_string(), "a".to_string()).into()],
                }
                .into()],
            ),
            Database::new(
                "d".to_string(),
                vec![Schema {
                    name: "e".to_string(),
                    tables: vec![Table::new_with_schema("f".to_string(), "d".to_string()).into()],
                }
                .into()],
            ),
        ];

        // a
        //   b
        //     c
        // d
        //   e
        //     f

        let mut tree = DatabaseTree::new(&items, &BTreeSet::new()).unwrap();
        tree.items.expand(0, false);
        tree.items.expand(1, false);
        tree.items.expand(3, false);
        tree.selection = Some(0);

        assert!(tree.move_selection(MoveSelection::Left));
        assert!(tree.move_selection(MoveSelection::Down));
        assert_eq!(tree.selection, Some(3));
        let s = tree.visual_selection().unwrap();

        assert_eq!(s.count, 4);
        assert_eq!(s.index, 1);
    }

    #[test]
    fn test_selection_top() {
        let items = vec![Database::new(
            "a".to_string(),
            vec![
                Table::new("b".to_string()).into(),
                Table::new("c".to_string()).into(),
                Table::new("d".to_string()).into(),
            ],
        )];

        // a
        //   b
        //   c
        //   d

        let mut tree = DatabaseTree::new(&items, &BTreeSet::new()).unwrap();
        tree.selection = Some(3);
        tree.items.expand(0, false);

        assert!(tree.move_selection(MoveSelection::Top));
        assert_eq!(tree.selection, Some(0));

        let items = vec![Database::new(
            "a".to_string(),
            vec![Schema {
                name: "b".to_string(),
                tables: vec![
                    Table::new("c".to_string()).into(),
                    Table::new("d".to_string()).into(),
                ],
            }
            .into()],
        )];

        // a
        //   b
        //     c
        //     d

        let mut tree = DatabaseTree::new(&items, &BTreeSet::new()).unwrap();
        tree.selection = Some(3);
        tree.items.expand(0, false);
        tree.items.expand(1, false);

        assert!(tree.move_selection(MoveSelection::Top));
        assert_eq!(tree.selection, Some(0));
    }

    #[test]
    fn test_selection_bottom() {
        let items = vec![Database::new(
            "a".to_string(),
            vec![
                Table::new("b".to_string()).into(),
                Table::new("c".to_string()).into(),
                Table::new("d".to_string()).into(),
            ],
        )];

        // a
        //   b
        //   c
        //   d

        let mut tree = DatabaseTree::new(&items, &BTreeSet::new()).unwrap();
        tree.selection = Some(0);
        tree.items.expand(0, false);

        assert!(tree.move_selection(MoveSelection::End));
        assert_eq!(tree.selection, Some(3));

        let items = vec![Database::new(
            "a".to_string(),
            vec![Schema {
                name: "b".to_string(),
                tables: vec![
                    Table::new("c".to_string()).into(),
                    Table::new("d".to_string()).into(),
                ],
            }
            .into()],
        )];

        // a
        //   b
        //     c
        //     d

        let mut tree = DatabaseTree::new(&items, &BTreeSet::new()).unwrap();
        tree.selection = Some(0);
        tree.items.expand(0, false);
        tree.items.expand(1, false);

        assert!(tree.move_selection(MoveSelection::End));
        assert_eq!(tree.selection, Some(3));
    }
}
