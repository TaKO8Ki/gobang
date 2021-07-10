use crate::error::Result;
use crate::{Database, Table};
use std::convert::TryFrom;

#[derive(Debug, Clone)]
pub struct TreeItemInfo {
    indent: u8,
    visible: bool,
}

impl TreeItemInfo {
    pub const fn new(indent: u8, visible: bool) -> Self {
        Self { indent, visible }
    }

    pub const fn is_visible(&self) -> bool {
        self.visible
    }

    pub const fn indent(&self) -> u8 {
        self.indent
    }

    pub fn unindent(&mut self) {
        self.indent = self.indent.saturating_sub(1);
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
}

/// `DatabaseTreeItem` can be of two kinds
#[derive(PartialEq, Debug, Clone)]
pub enum DatabaseTreeItemKind {
    Database { name: String, collapsed: bool },
    Table { database: String, table: Table },
}

impl DatabaseTreeItemKind {
    pub const fn is_database(&self) -> bool {
        matches!(self, Self::Database { .. })
    }

    pub const fn is_table(&self) -> bool {
        matches!(self, Self::Table { .. })
    }

    pub const fn is_database_collapsed(&self) -> bool {
        match self {
            Self::Database { collapsed, .. } => *collapsed,
            Self::Table { .. } => false,
        }
    }

    pub fn name(&self) -> String {
        match self {
            Self::Database { name, .. } => name.to_string(),
            Self::Table { table, .. } => table.name.clone(),
        }
    }

    pub fn database_name(&self) -> Option<String> {
        match self {
            Self::Database { .. } => None,
            Self::Table { database, .. } => Some(database.clone()),
        }
    }
}

/// `DatabaseTreeItem` can be of two kinds: see `DatabaseTreeItem` but shares an info
#[derive(Debug, Clone)]
pub struct DatabaseTreeItem {
    info: TreeItemInfo,
    kind: DatabaseTreeItemKind,
}

impl DatabaseTreeItem {
    pub fn new_table(database: &Database, table: &Table) -> Result<Self> {
        let indent = u8::try_from((3_usize).saturating_sub(2))?;

        Ok(Self {
            info: TreeItemInfo::new(indent, false),
            kind: DatabaseTreeItemKind::Table {
                database: database.name.clone(),
                table: table.clone(),
            },
        })
    }

    pub fn new_database(database: &Database, _collapsed: bool) -> Result<Self> {
        Ok(Self {
            info: TreeItemInfo::new(0, true),
            kind: DatabaseTreeItemKind::Database {
                name: database.name.to_string(),
                collapsed: true,
            },
        })
    }

    pub fn set_collapsed(&mut self, collapsed: bool) {
        if let DatabaseTreeItemKind::Database { name, .. } = self.kind() {
            self.kind = DatabaseTreeItemKind::Database {
                name: name.to_string(),
                collapsed,
            }
        }
    }

    pub const fn info(&self) -> &TreeItemInfo {
        &self.info
    }

    pub fn info_mut(&mut self) -> &mut TreeItemInfo {
        &mut self.info
    }

    pub const fn kind(&self) -> &DatabaseTreeItemKind {
        &self.kind
    }

    pub fn collapse_database(&mut self) {
        if let DatabaseTreeItemKind::Database { name, .. } = &self.kind {
            self.kind = DatabaseTreeItemKind::Database {
                name: name.to_string(),
                collapsed: true,
            }
        }
    }

    pub fn expand_database(&mut self) {
        if let DatabaseTreeItemKind::Database { name, .. } = &self.kind {
            self.kind = DatabaseTreeItemKind::Database {
                name: name.to_string(),
                collapsed: false,
            };
        }
    }

    pub fn show(&mut self) {
        self.info.visible = true;
    }

    pub fn hide(&mut self) {
        self.info.visible = false;
    }

    pub fn is_match(&self, filter_text: &String) -> bool {
        match self.kind.clone() {
            DatabaseTreeItemKind::Database { name, .. } => name.contains(filter_text),
            DatabaseTreeItemKind::Table { table, .. } => table.name.contains(filter_text),
        }
    }

    pub fn is_database(&self) -> bool {
        self.kind.is_database()
    }
}

impl Eq for DatabaseTreeItem {}

impl PartialEq for DatabaseTreeItem {
    fn eq(&self, other: &Self) -> bool {
        if self.kind.is_database() && other.kind().is_database() {
            return self.kind.name().eq(&other.kind.name());
        }
        if !self.kind.is_database() && !other.kind.is_database() {
            return self.kind.name().eq(&other.kind.name());
        }
        false
    }
}

impl PartialOrd for DatabaseTreeItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.kind.name().partial_cmp(&other.kind.name())
    }
}

impl Ord for DatabaseTreeItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.kind.name().cmp(&other.kind.name())
    }
}
