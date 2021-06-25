use crate::{
    user_config::{Connection, UserConfig},
    utils::get_tables,
};
use sqlx::mysql::MySqlPool;
use tui::widgets::{ListState, TableState};

pub enum InputMode {
    Normal,
    Editing,
}

pub enum FocusBlock {
    DabataseList(bool),
    TableList(bool),
    RecordTable(bool),
    ConnectionList,
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
    pub engine: String,
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
    pub input_mode: InputMode,
    pub query: String,
    pub databases: Vec<Database>,
    pub record_table: RecordTable,
    pub focus_type: FocusBlock,
    pub user_config: Option<UserConfig>,
    pub selected_connection: ListState,
    pub selected_database: ListState,
    pub selected_table: ListState,
    pub pool: Option<MySqlPool>,
}

impl Default for App {
    fn default() -> App {
        App {
            input: String::new(),
            input_mode: InputMode::Normal,
            query: String::new(),
            databases: Vec::new(),
            record_table: RecordTable::default(),
            focus_type: FocusBlock::DabataseList(false),
            user_config: None,
            selected_connection: ListState::default(),
            selected_database: ListState::default(),
            selected_table: ListState::default(),
            pool: None,
        }
    }
}

impl App {
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

    pub fn selected_database(&self) -> Option<&Database> {
        match self.selected_database.selected() {
            Some(i) => match self.databases.get(i) {
                Some(db) => Some(db),
                None => None,
            },
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
}
