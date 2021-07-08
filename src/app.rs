use crate::components::DrawableComponent as _;
use crate::{
    components::tab::Tab,
    components::{
        ConnectionsComponent, DatabasesComponent, QueryComponent, TabComponent, TableComponent,
        TableStatusComponent,
    },
    user_config::UserConfig,
};
use sqlx::MySqlPool;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, ListState, Paragraph},
    Frame,
};

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
    pub table_status: TableStatusComponent,
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
            table_status: TableStatusComponent::default(),
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

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<'_, B>) -> anyhow::Result<()> {
        if let FocusBlock::ConnectionList = self.focus_block {
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
            .draw(
                f,
                left_chunks[0],
                matches!(self.focus_block, FocusBlock::DabataseList),
            )
            .unwrap();
        self.table_status.draw(
            f,
            left_chunks[1],
            matches!(self.focus_block, FocusBlock::DabataseList),
        )?;

        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(5),
                ]
                .as_ref(),
            )
            .split(main_chunks[1]);

        self.tab.draw(f, right_chunks[0], false)?;
        self.query.draw(
            f,
            right_chunks[1],
            matches!(self.focus_block, FocusBlock::Query),
        )?;

        match self.tab.selected_tab {
            Tab::Records => self.record_table.draw(
                f,
                right_chunks[2],
                matches!(self.focus_block, FocusBlock::Table),
            )?,
            Tab::Structure => self.structure_table.draw(
                f,
                right_chunks[2],
                matches!(self.focus_block, FocusBlock::Table),
            )?,
        }
        self.draw_error_popup(f)?;
        Ok(())
    }

    fn draw_error_popup<B: Backend>(&self, f: &mut Frame<'_, B>) -> anyhow::Result<()> {
        if let Some(error) = self.error.as_ref() {
            let percent_x = 60;
            let percent_y = 20;
            let error = Paragraph::new(error.to_string())
                .block(Block::default().title("Error").borders(Borders::ALL))
                .style(Style::default().fg(Color::Red));
            let popup_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Percentage((100 - percent_y) / 2),
                        Constraint::Percentage(percent_y),
                        Constraint::Percentage((100 - percent_y) / 2),
                    ]
                    .as_ref(),
                )
                .split(f.size());

            let area = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Percentage((100 - percent_x) / 2),
                        Constraint::Percentage(percent_x),
                        Constraint::Percentage((100 - percent_x) / 2),
                    ]
                    .as_ref(),
                )
                .split(popup_layout[1])[1];
            f.render_widget(Clear, area);
            f.render_widget(error, area);
        }
        Ok(())
    }
}
