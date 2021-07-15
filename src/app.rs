use crate::clipboard::Clipboard;
use crate::components::DrawableComponent as _;
use crate::{
    components::tab::Tab,
    components::{
        ConnectionsComponent, DatabasesComponent, ErrorComponent, RecordTableComponent,
        TabComponent, TableComponent, TableStatusComponent,
    },
    user_config::UserConfig,
};
use sqlx::MySqlPool;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::ListState,
    Frame,
};

pub enum Focus {
    DabataseList,
    Table,
    ConnectionList,
}
pub struct App {
    pub record_table: RecordTableComponent,
    pub structure_table: TableComponent,
    pub focus: Focus,
    pub tab: TabComponent,
    pub user_config: Option<UserConfig>,
    pub selected_connection: ListState,
    pub databases: DatabasesComponent,
    pub connections: ConnectionsComponent,
    pub table_status: TableStatusComponent,
    pub clipboard: Clipboard,
    pub pool: Option<MySqlPool>,
    pub error: ErrorComponent,
}

impl Default for App {
    fn default() -> App {
        App {
            record_table: RecordTableComponent::default(),
            structure_table: TableComponent::default(),
            focus: Focus::DabataseList,
            tab: TabComponent::default(),
            user_config: None,
            selected_connection: ListState::default(),
            databases: DatabasesComponent::new(),
            connections: ConnectionsComponent::default(),
            table_status: TableStatusComponent::default(),
            clipboard: Clipboard::new(),
            pool: None,
            error: ErrorComponent::default(),
        }
    }
}

impl App {
    pub fn new(user_config: UserConfig) -> App {
        App {
            user_config: Some(user_config.clone()),
            connections: ConnectionsComponent::new(user_config.conn),
            focus: Focus::ConnectionList,
            ..App::default()
        }
    }

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<'_, B>) -> anyhow::Result<()> {
        if let Focus::ConnectionList = self.focus {
            self.connections.draw(
                f,
                Layout::default()
                    .constraints([Constraint::Percentage(100)])
                    .split(f.size())[0],
                false,
            )?;
            return Ok(());
        }

        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(15), Constraint::Percentage(85)])
            .split(f.size());
        let left_chunks = Layout::default()
            .constraints([Constraint::Min(8), Constraint::Length(7)].as_ref())
            .split(main_chunks[0]);

        self.databases
            .draw(f, left_chunks[0], matches!(self.focus, Focus::DabataseList))
            .unwrap();
        self.table_status
            .draw(f, left_chunks[1], matches!(self.focus, Focus::DabataseList))?;

        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Length(5)].as_ref())
            .split(main_chunks[1]);

        self.tab.draw(f, right_chunks[0], false)?;

        match self.tab.selected_tab {
            Tab::Records => {
                self.record_table
                    .draw(f, right_chunks[1], matches!(self.focus, Focus::Table))?
            }
            Tab::Structure => {
                self.structure_table
                    .draw(f, right_chunks[1], matches!(self.focus, Focus::Table))?
            }
        }
        self.error.draw(f, Rect::default(), false)?;
        Ok(())
    }
}
