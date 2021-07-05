use crate::{
    user_config::{Connection, UserConfig},
    utils::get_tables,
};
use sqlx::mysql::MySqlPool;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use tui::widgets::{ListState, TableState};
use unicode_width::UnicodeWidthStr;

#[derive(Debug, Clone, Copy, EnumIter)]
pub enum Tab {
    Records,
    Structure,
}

impl std::fmt::Display for Tab {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Tab {
    pub fn names() -> Vec<String> {
        Self::iter()
            .map(|tab| format!("{} [{}]", tab, tab as u8 + 1))
            .collect()
    }
}

pub enum FocusBlock {
    DabataseList(bool),
    TableList(bool),
    RecordTable(bool),
    ConnectionList,
    Query(bool),
}

#[derive(Clone)]
pub struct Database {
    pub name: String,
    pub tables: Vec<Table>,
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct Table {
    #[sqlx(rename = "Name")]
    pub name: String,
    #[sqlx(rename = "Create_time")]
    pub create_time: chrono::DateTime<chrono::Utc>,
    #[sqlx(rename = "Update_time")]
    pub update_time: Option<chrono::DateTime<chrono::Utc>>,
    #[sqlx(rename = "Engine")]
    pub engine: Option<String>,
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct Column {
    #[sqlx(rename = "Field")]
    pub field: String,
    #[sqlx(rename = "Type")]
    pub r#type: String,
    #[sqlx(rename = "Collation")]
    pub collation: String,
    #[sqlx(rename = "Null")]
    pub null: String,
}

pub struct RecordTable {
    pub state: TableState,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub column_index: usize,
}

impl Default for RecordTable {
    fn default() -> Self {
        Self {
            state: TableState::default(),
            headers: vec![],
            rows: vec![],
            column_index: 0,
        }
    }
}

impl RecordTable {
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.rows.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.rows.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn next_column(&mut self) {
        if self.headers.len() > 9 && self.column_index < self.headers.len() - 9 {
            self.column_index += 1
        }
    }

    pub fn previous_column(&mut self) {
        if self.column_index > 0 {
            self.column_index -= 1
        }
    }

    pub fn headers(&self) -> Vec<String> {
        let mut headers = self.headers[self.column_index..].to_vec();
        headers.insert(0, "".to_string());
        headers
    }

    pub fn rows(&self) -> Vec<Vec<String>> {
        let mut rows = self
            .rows
            .iter()
            .map(|row| row[self.column_index..].to_vec())
            .collect::<Vec<Vec<String>>>();
        for (index, row) in rows.iter_mut().enumerate() {
            row.insert(0, (index + 1).to_string())
        }
        rows
    }
}

impl Database {
    pub async fn new(name: String, pool: &MySqlPool) -> anyhow::Result<Self> {
        Ok(Self {
            name: name.clone(),
            tables: get_tables(name, pool).await?,
        })
    }
}

pub struct App {
    pub input: String,
    pub input_cursor_x: u16,
    pub query: String,
    pub databases: Vec<Database>,
    pub record_table: RecordTable,
    pub structure_table: RecordTable,
    pub focus_block: FocusBlock,
    pub selected_tab: Tab,
    pub user_config: Option<UserConfig>,
    pub selected_connection: ListState,
    pub selected_database: ListState,
    pub selected_table: ListState,
    pub revision_files: crate::components::RevisionFilesComponent,
    pub pool: Option<MySqlPool>,
    pub error: Option<String>,
}

impl Default for App {
    fn default() -> App {
        App {
            input: String::new(),
            input_cursor_x: 0,
            query: String::new(),
            databases: Vec::new(),
            record_table: RecordTable::default(),
            structure_table: RecordTable::default(),
            focus_block: FocusBlock::DabataseList(false),
            selected_tab: Tab::Records,
            user_config: None,
            selected_connection: ListState::default(),
            selected_database: ListState::default(),
            selected_table: ListState::default(),
            revision_files: crate::components::RevisionFilesComponent::new(),
            pool: None,
            error: None,
        }
    }
}

impl App {
    pub fn next_tab(&mut self) {
        self.selected_tab = match self.selected_tab {
            Tab::Records => Tab::Structure,
            Tab::Structure => Tab::Records,
        }
    }

    pub fn previous_tab(&mut self) {
        self.selected_tab = match self.selected_tab {
            Tab::Records => Tab::Structure,
            Tab::Structure => Tab::Records,
        }
    }

    pub fn next_table(&mut self) {
        let i = match self.selected_table.selected() {
            Some(i) => {
                if i >= self.selected_database().unwrap().tables.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.selected_table.select(Some(i));
    }

    pub fn previous_table(&mut self) {
        let i = match self.selected_table.selected() {
            Some(i) => {
                if i == 0 {
                    self.selected_database().unwrap().tables.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.selected_table.select(Some(i));
    }

    pub fn next_database(&mut self) {
        let i = match self.selected_database.selected() {
            Some(i) => {
                if i >= self.databases.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.selected_table.select(Some(0));
        self.selected_database.select(Some(i));
    }

    pub fn previous_database(&mut self) {
        let i = match self.selected_database.selected() {
            Some(i) => {
                if i == 0 {
                    self.databases.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.selected_table.select(Some(0));
        self.selected_database.select(Some(i));
    }

    pub fn next_connection(&mut self) {
        if let Some(config) = &self.user_config {
            let i = match self.selected_connection.selected() {
                Some(i) => {
                    if i >= config.conn.len() - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            self.selected_connection.select(Some(i));
        }
    }

    pub fn previous_connection(&mut self) {
        if let Some(config) = &self.user_config {
            let i = match self.selected_connection.selected() {
                Some(i) => {
                    if i == 0 {
                        config.conn.len() - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.selected_connection.select(Some(i));
        }
    }

    pub fn increment_input_cursor_x(&mut self) {
        if self.input_cursor_x > 0 {
            self.input_cursor_x -= 1;
        }
    }

    pub fn decrement_input_cursor_x(&mut self) {
        if self.input_cursor_x < self.input.width() as u16 {
            self.input_cursor_x += 1;
        }
    }

    pub fn selected_database(&self) -> Option<&Database> {
        match self.selected_database.selected() {
            Some(i) => self.databases.get(i),
            None => None,
        }
    }

    pub fn selected_table(&self) -> Option<&Table> {
        match self.selected_table.selected() {
            Some(i) => match self.selected_database() {
                Some(db) => db.tables.get(i),
                None => None,
            },
            None => None,
        }
    }

    pub fn selected_connection(&self) -> Option<&Connection> {
        match &self.user_config {
            Some(config) => match self.selected_connection.selected() {
                Some(i) => config.conn.get(i),
                None => None,
            },
            None => None,
        }
    }

    pub fn table_status(&self) -> Vec<String> {
        if let Some(table) = self.selected_table() {
            return vec![
                format!("created: {}", table.create_time.to_string()),
                format!(
                    "updated: {}",
                    table
                        .update_time
                        .map(|time| time.to_string())
                        .unwrap_or_default()
                ),
                format!(
                    "engine: {}",
                    table
                        .engine
                        .as_ref()
                        .map(|engine| engine.to_string())
                        .unwrap_or_default()
                ),
                format!("rows: {}", self.record_table.rows.len()),
            ];
        }
        Vec::new()
    }
}
