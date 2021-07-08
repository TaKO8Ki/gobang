use crate::{
    components::{
        ConnectionsComponent, DatabasesComponent, QueryComponent, TabComponent, TableComponent,
    },
    user_config::UserConfig,
};
use sqlx::mysql::MySqlPool;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use tui::widgets::ListState;

pub enum FocusBlock {
    DabataseList,
    Table,
    ConnectionList,
    Query,
}
pub struct App {
    pub query: QueryComponent,
    pub record_table: TableComponent,
    pub structure_table: TableComponent,
    pub focus_block: FocusBlock,
    pub tab: TabComponent,
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
            tab: TabComponent::default(),
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
