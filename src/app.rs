use crate::clipboard::Clipboard;
use crate::components::{CommandInfo, Component as _, DrawableComponent as _, EventState};
use crate::event::Key;
use crate::utils::{MySqlPool, Pool};
use crate::{
    components::tab::Tab,
    components::{
        ConnectionsComponent, DatabasesComponent, ErrorComponent, HelpComponent,
        RecordTableComponent, TabComponent, TableComponent, TableStatusComponent,
    },
    config::Config,
};
use database_tree::Database;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

pub enum Focus {
    DabataseList,
    Table,
    ConnectionList,
}
pub struct App {
    record_table: RecordTableComponent,
    structure_table: TableComponent,
    focus: Focus,
    tab: TabComponent,
    help: HelpComponent,
    databases: DatabasesComponent,
    connections: ConnectionsComponent,
    table_status: TableStatusComponent,
    clipboard: Clipboard,
    pool: Option<Box<dyn Pool>>,
    pub config: Config,
    pub error: ErrorComponent,
}

impl App {
    pub fn new(config: Config) -> App {
        Self {
            focus: Focus::ConnectionList,
            config: config.clone(),
            connections: ConnectionsComponent::new(config.key_config.clone(), config.conn),
            record_table: RecordTableComponent::default(),
            structure_table: TableComponent::default(),
            tab: TabComponent::default(),
            help: HelpComponent::new(config.key_config.clone()),
            databases: DatabasesComponent::new(config.key_config.clone()),
            table_status: TableStatusComponent::default(),
            clipboard: Clipboard::new(),
            pool: None,
            error: ErrorComponent::new(config.key_config.clone()),
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
        self.help.draw(f, Rect::default(), false)?;
        Ok(())
    }

    fn update_commands(&mut self) {
        self.help.set_cmds(self.commands());
    }

    fn commands(&self) -> Vec<CommandInfo> {
        let res = vec![
            CommandInfo::new(crate::components::command::move_left("h"), true, true),
            CommandInfo::new(crate::components::command::move_down("j"), true, true),
            CommandInfo::new(crate::components::command::move_up("k"), true, true),
            CommandInfo::new(crate::components::command::move_right("l"), true, true),
            CommandInfo::new(crate::components::command::filter("/"), true, true),
            CommandInfo::new(
                crate::components::command::move_focus_to_right_widget(
                    Key::Right.to_string().as_str(),
                ),
                true,
                true,
            ),
        ];

        res
    }

    pub async fn event(&mut self, key: Key) -> anyhow::Result<EventState> {
        self.update_commands();

        if self.components_event(key).await?.is_consumed() {
            return Ok(EventState::Consumed);
        };

        if self.move_focus(key)?.is_consumed() {
            return Ok(EventState::Consumed);
        };
        Ok(EventState::NotConsumed)
    }

    pub async fn components_event(&mut self, key: Key) -> anyhow::Result<EventState> {
        if self.error.event(key)?.is_consumed() {
            return Ok(EventState::Consumed);
        }

        if self.help.event(key)?.is_consumed() {
            return Ok(EventState::Consumed);
        }

        match self.focus {
            Focus::ConnectionList => {
                if self.connections.event(key)?.is_consumed() {
                    return Ok(EventState::Consumed);
                }

                if key == self.config.key_config.enter {
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

                if key == self.config.key_config.enter && self.databases.tree_focused() {
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

                        if key == self.config.key_config.copy {
                            if let Some(text) = self.record_table.table.selected_cells() {
                                self.clipboard.store(text)
                            }
                        }

                        if key == self.config.key_config.enter && self.record_table.filter_focused()
                        {
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

                        if let Some(index) = self.record_table.table.selected_row.selected() {
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

                        if key == self.config.key_config.copy {
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
                if key == self.config.key_config.enter {
                    self.focus = Focus::DabataseList;
                    return Ok(EventState::Consumed);
                }
            }
            Focus::DabataseList => {
                if key == self.config.key_config.focus_right && self.databases.tree_focused() {
                    self.focus = Focus::Table;
                    return Ok(EventState::Consumed);
                }
            }
            Focus::Table => {
                if key == self.config.key_config.focus_left {
                    self.focus = Focus::DabataseList;
                    return Ok(EventState::Consumed);
                }
            }
        }
        Ok(EventState::NotConsumed)
    }
}
