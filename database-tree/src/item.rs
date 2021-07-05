use crate::error::Result;
use std::{convert::TryFrom, path::PathBuf};

/// holds the information shared among all `DatabaseTreeItem` in a `FileTree`
#[derive(Debug, Clone)]
pub struct TreeItemInfo {
    /// indent level
    indent: u8,
    /// currently visible depending on the folder collapse states
    visible: bool,
    /// contains this paths last component and folded up paths added to it
    /// if this is `None` nothing was folding into here
    folded: Option<PathBuf>,
    /// the full path
    pub full_path: String,
    pub database: Option<String>,
}

impl TreeItemInfo {
    ///
    pub const fn new(indent: u8, database: Option<String>, full_path: String) -> Self {
        Self {
            indent,
            visible: true,
            folded: None,
            full_path,
            database,
        }
    }

    ///
    pub const fn is_visible(&self) -> bool {
        self.visible
    }

    ///
    //TODO: remove
    pub fn full_path_str(&self) -> &str {
        self.full_path.as_str()
    }

    // ///
    // pub fn full_path(&self) -> &Path {
    //     self.full_path.as_path()
    // }

    /// like `path` but as `&str`
    pub fn path_str(&self) -> &str {
        match self.full_path.split('/').collect::<Vec<_>>().get(1) {
            Some(path) => path,
            None => self.full_path.as_str(),
        }
    }

    /// returns the last component of `full_path`
    /// or the last components plus folded up children paths
    // pub fn path(&self) -> &Path {
    //     self.folded.as_ref().map_or_else(
    //         || {
    //             Path::new(
    //                 self.full_path
    //                     .components()
    //                     .last()
    //                     .and_then(|c| c.as_os_str().to_str())
    //                     .unwrap_or_default(),
    //             )
    //         },
    //         |folding| folding.as_path(),
    //     )
    // }

    ///
    pub const fn indent(&self) -> u8 {
        self.indent
    }

    ///
    pub fn unindent(&mut self) {
        self.indent = self.indent.saturating_sub(1);
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
}

/// attribute used to indicate the collapse/expand state of a path item
#[derive(PartialEq, Debug, Copy, Clone)]
pub struct DatabaseCollapsed(pub bool);

/// `DatabaseTreeItem` can be of two kinds
#[derive(PartialEq, Debug, Clone)]
pub enum DatabaseTreeItemKind {
    Database(DatabaseCollapsed),
    File,
}

impl DatabaseTreeItemKind {
    pub const fn is_database(&self) -> bool {
        matches!(self, Self::Database(_))
    }

    pub const fn is_database_collapsed(&self) -> bool {
        match self {
            Self::Database(collapsed) => collapsed.0,
            Self::File => false,
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
    pub fn new_table(database: String, path: String) -> Result<Self> {
        let indent = u8::try_from((3 as usize).saturating_sub(2))?;

        Ok(Self {
            info: TreeItemInfo::new(indent, Some(database), path),
            kind: DatabaseTreeItemKind::File,
        })
    }

    pub fn new_database(path: String, collapsed: bool) -> Result<Self> {
        Ok(Self {
            info: TreeItemInfo::new(0, None, path),
            kind: DatabaseTreeItemKind::Database(DatabaseCollapsed(collapsed)),
        })
    }

    ///
    pub fn fold(&mut self, next: Self) {
        if let Some(folded) = self.info.folded.as_mut() {
            *folded = folded.join(&next.info.full_path);
        } else {
            // self.info.folded = Some(self.info.full_path.join(next.info.full_path));
        }

        self.info.full_path = next.info.full_path.clone();
    }

    ///
    pub const fn info(&self) -> &TreeItemInfo {
        &self.info
    }

    ///
    pub fn info_mut(&mut self) -> &mut TreeItemInfo {
        &mut self.info
    }

    ///
    pub const fn kind(&self) -> &DatabaseTreeItemKind {
        &self.kind
    }

    ///
    pub fn collapse_database(&mut self) {
        self.kind = DatabaseTreeItemKind::Database(DatabaseCollapsed(true));
    }

    ///
    pub fn expand_database(&mut self) {
        self.kind = DatabaseTreeItemKind::Database(DatabaseCollapsed(false));
    }

    ///
    pub fn hide(&mut self) {
        self.info.visible = false;
    }
}

impl Eq for DatabaseTreeItem {}

impl PartialEq for DatabaseTreeItem {
    fn eq(&self, other: &Self) -> bool {
        self.info.full_path.eq(&other.info.full_path)
    }
}

impl PartialOrd for DatabaseTreeItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.info.full_path.partial_cmp(&other.info.full_path)
    }
}

impl Ord for DatabaseTreeItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.info.full_path.cmp(&other.info.full_path)
    }
}
