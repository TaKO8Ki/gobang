use crate::clipboard::Clipboard;
use crate::components::Component as _;
use crate::components::DrawableComponent as _;
use crate::components::EventState;
use crate::event::Key;
use crate::utils::{MySqlPool, Pool};
use crate::{
    components::tab::Tab,
    components::{
        ConnectionsComponent, DatabasesComponent, ErrorComponent, RecordTableComponent,
        TabComponent, TableComponent, TableStatusComponent,
    },
    user_config::UserConfig,
};
use database_tree::Database;
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
    pub pool: Option<Box<dyn Pool>>,
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

    pub async fn event(&mut self, key: Key) -> anyhow::Result<EventState> {
        if let Key::Esc = key {
            if self.error.error.is_some() {
                self.error.error = None;
                return Ok(EventState::Consumed);
            }
        }

        if self.components_event(key).await?.is_consumed() {
            return Ok(EventState::Consumed);
        };

        if self.move_focus(key)?.is_consumed() {
            return Ok(EventState::Consumed);
        };
        Ok(EventState::NotConsumed)
    }

    pub async fn components_event(&mut self, key: Key) -> anyhow::Result<EventState> {
        match self.focus {
            Focus::ConnectionList => {
                if self.connections.event(key)?.is_consumed() {
                    return Ok(EventState::Consumed);
                }

                if let Key::Enter = key {
                    self.record_table.reset();
                    if let Some(conn) = self.connections.selected_connection() {
                        if let Some(pool) = self.pool.as_ref() {
                            pool.close().await;
                        }
                        self.pool = Some(Box::new(
                            MySqlPool::new(conn.database_url().as_str()).await?,
                        ));
                        let databases = match &conn.database {
                            Some(database) => vec![Database::new(
                                database.clone(),
                                self.pool
                                    .as_ref()
                                    .unwrap()
                                    .get_tables(database.clone())
                                    .await?,
                            )],
                            None => self.pool.as_ref().unwrap().get_databases().await?,
                        };
                        self.databases.update(databases.as_slice()).unwrap();
                        self.focus = Focus::DabataseList
                    }
                    return Ok(EventState::Consumed);
                }
            }
            Focus::DabataseList => {
                if self.databases.event(key)?.is_consumed() {
                    return Ok(EventState::Consumed);
                }

                if matches!(key, Key::Enter) && self.databases.tree_focused() {
                    if let Some((table, database)) = self.databases.tree().selected_table() {
                        self.focus = Focus::Table;
                        let (headers, records) = self
                            .pool
                            .as_ref()
                            .unwrap()
                            .get_records(&database, &table.name, 0, None)
                            .await?;
                        self.record_table = RecordTableComponent::new(records, headers);
                        self.record_table.set_table(table.name.to_string());

                        let (headers, records) = self
                            .pool
                            .as_ref()
                            .unwrap()
                            .get_columns(&database, &table.name)
                            .await?;
                        self.structure_table = TableComponent::new(records, headers);

                        self.table_status
                            .update(self.record_table.len() as u64, table);
                    }
                    return Ok(EventState::Consumed);
                }
            }
            Focus::Table => {
                match self.tab.selected_tab {
                    Tab::Records => {
                        if self.record_table.event(key)?.is_consumed() {
                            return Ok(EventState::Consumed);
                        };

                        if let Key::Char('y') = key {
                            if let Some(text) = self.record_table.table.selected_cells() {
                                self.clipboard.store(text)
                            }
                        }

                        if matches!(key, Key::Enter) && self.record_table.filter_focused() {
                            self.record_table.focus = crate::components::record_table::Focus::Table;
                            if let Some((table, database)) = self.databases.tree().selected_table()
                            {
                                let (headers, records) = self
                                    .pool
                                    .as_ref()
                                    .unwrap()
                                    .get_records(
                                        &database.clone(),
                                        &table.name,
                                        0,
                                        if self.record_table.filter.input.is_empty() {
                                            None
                                        } else {
                                            Some(self.record_table.filter.input_str())
                                        },
                                    )
                                    .await?;
                                self.record_table.update(records, headers);
                            }
                        }

                        if self.record_table.table.eod {
                            return Ok(EventState::Consumed);
                        }

                        if let Some(index) = self.record_table.table.state.selected() {
                            if index.saturating_add(1)
                                % crate::utils::RECORDS_LIMIT_PER_PAGE as usize
                                == 0
                            {
                                if let Some((table, database)) =
                                    self.databases.tree().selected_table()
                                {
                                    let (_, records) = self
                                        .pool
                                        .as_ref()
                                        .unwrap()
                                        .get_records(
                                            &database.clone(),
                                            &table.name,
                                            index as u16,
                                            if self.record_table.filter.input.is_empty() {
                                                None
                                            } else {
                                                Some(self.record_table.filter.input_str())
                                            },
                                        )
                                        .await?;
                                    if !records.is_empty() {
                                        self.record_table.table.rows.extend(records);
                                    } else {
                                        self.record_table.table.end()
                                    }
                                }
                            }
                        };
                    }
                    Tab::Structure => {
                        if self.structure_table.event(key)?.is_consumed() {
                            return Ok(EventState::Consumed);
                        };

                        if let Key::Char('y') = key {
                            if let Some(text) = self.structure_table.selected_cells() {
                                self.clipboard.store(text)
                            }
                        };
                    }
                };
            }
        }
        Ok(EventState::NotConsumed)
    }

    pub fn move_focus(&mut self, key: Key) -> anyhow::Result<EventState> {
        if let Key::Char('c') = key {
            self.focus = Focus::ConnectionList;
            return Ok(EventState::Consumed);
        }
        if self.tab.event(key)?.is_consumed() {
            return Ok(EventState::Consumed);
        }
        match self.focus {
            Focus::ConnectionList => {
                if let Key::Enter = key {
                    self.focus = Focus::DabataseList;
                    return Ok(EventState::Consumed);
                }
            }
            Focus::DabataseList => match key {
                Key::Right if self.databases.tree_focused() => {
                    self.focus = Focus::Table;
                    return Ok(EventState::Consumed);
                }
                _ => (),
            },
            Focus::Table => match key {
                Key::Left => {
                    self.focus = Focus::DabataseList;
                    return Ok(EventState::Consumed);
                }
                _ => (),
            },
        }
        Ok(EventState::NotConsumed)
    }
}
