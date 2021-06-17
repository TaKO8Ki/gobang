use sqlx::mysql::MySqlPool;
use sqlx::Row;
use tui::widgets::List;
use tui::widgets::{ListState, TableState};

pub enum InputMode {
    Normal,
    Editing,
}

pub enum FocusType {
    Dabatases(bool),
    Tables(bool),
    Records(bool),
}

#[derive(Clone)]
pub struct Database {
    pub selected_table: ListState,
    pub name: String,
    pub tables: Vec<Table>,
}

#[derive(Clone)]
pub struct Table {
    pub name: String,
}

impl Database {
    pub async fn new(name: String, pool: &MySqlPool) -> anyhow::Result<Self> {
        let tables = sqlx::query(format!("show tables from {}", name).as_str())
            .fetch_all(pool)
            .await?
            .iter()
            .map(|table| Table { name: table.get(0) })
            .collect::<Vec<Table>>();
        Ok(Self {
            selected_table: ListState::default(),
            name,
            tables,
        })
    }

    pub fn next(&mut self) {
        let i = match self.selected_table.selected() {
            Some(i) => {
                if i >= self.tables.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.selected_table.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.selected_table.selected() {
            Some(i) => {
                if i == 0 {
                    self.tables.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.selected_table.select(Some(i));
    }
}

pub struct App {
    pub input: String,
    pub input_mode: InputMode,
    pub messages: Vec<Vec<String>>,
    pub selected_database: ListState,
    pub databases: Vec<Database>,
    pub focus_type: FocusType,
}

impl Default for App {
    fn default() -> App {
        App {
            input: String::new(),
            input_mode: InputMode::Normal,
            messages: Vec::new(),
            selected_database: ListState::default(),
            databases: Vec::new(),
            focus_type: FocusType::Dabatases(false),
        }
    }
}

impl App {
    pub fn next(&mut self) {
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
        self.selected_database.select(Some(i));
    }

    pub fn previous(&mut self) {
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
        self.selected_database.select(Some(i));
    }
}
