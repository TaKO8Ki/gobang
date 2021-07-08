use crate::{
    components::{ConnectionsComponent, DatabasesComponent, QueryComponent, TableComponent},
    user_config::{Connection, UserConfig},
};
use sqlx::mysql::MySqlPool;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use tui::widgets::ListState;

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
    DabataseList,
    Table,
    ConnectionList,
    Query,
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
pub struct App {
    pub query: QueryComponent,
    pub record_table: TableComponent,
    pub structure_table: TableComponent,
    pub focus_block: FocusBlock,
    pub selected_tab: Tab,
    pub user_config: Option<UserConfig>,
    pub selected_connection: ListState,
    pub databases: DatabasesComponent,
    pub connections: ConnectionsComponent,
    pub pool: Option<MySqlPool>,
    pub error: Option<String>,
}

impl Default for App {
    fn default() -> App {
        App {
            query: QueryComponent::default(),
            record_table: TableComponent::default(),
            structure_table: TableComponent::default(),
            focus_block: FocusBlock::DabataseList,
            selected_tab: Tab::Records,
            user_config: None,
            selected_connection: ListState::default(),
            databases: DatabasesComponent::new(),
            connections: ConnectionsComponent::default(),
            pool: None,
            error: None,
        }
    }
}

impl App {
    pub fn new(user_config: UserConfig) -> App {
        App {
            user_config: Some(user_config.clone()),
            connections: ConnectionsComponent::new(user_config.conn),
            focus_block: FocusBlock::ConnectionList,
            ..App::default()
        }
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
        if let Some((table, _)) = self.databases.tree.selected_table() {
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
