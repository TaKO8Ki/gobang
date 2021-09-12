use super::{Component, DrawableComponent, EventState};
use crate::components::command::CommandInfo;
use crate::components::{TableComponent, TableFilterComponent};
use crate::config::KeyConfig;
use crate::event::Key;
use anyhow::Result;
use database_tree::{Database, Table as DTable};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

pub enum Focus {
    Table,
    Filter,
}

pub struct RecordTableComponent {
    pub filter: TableFilterComponent,
    pub table: TableComponent,
    pub focus: Focus,
    key_config: KeyConfig,
}

impl RecordTableComponent {
    pub fn new(key_config: KeyConfig) -> Self {
        Self {
            filter: TableFilterComponent::new(key_config.clone()),
            table: TableComponent::new(key_config.clone()),
            focus: Focus::Table,
            key_config,
        }
    }

    pub fn update(
        &mut self,
        rows: Vec<Vec<String>>,
        headers: Vec<String>,
        database: Database,
        table: DTable,
    ) {
        self.table.update(rows, headers, database, table.clone());
        self.filter.table = Some(table);
    }

    pub fn reset(&mut self) {
        self.table.reset();
        self.filter.reset();
    }

    pub fn filter_focused(&self) -> bool {
        matches!(self.focus, Focus::Filter)
    }
}

impl DrawableComponent for RecordTableComponent {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect, focused: bool) -> Result<()> {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(3), Constraint::Length(5)])
            .split(area);

        self.table
            .draw(f, layout[1], focused && matches!(self.focus, Focus::Table))?;

        self.filter
            .draw(f, layout[0], focused && matches!(self.focus, Focus::Filter))?;
        Ok(())
    }
}

impl Component for RecordTableComponent {
    fn commands(&self, out: &mut Vec<CommandInfo>) {
        self.table.commands(out)
    }

    fn event(&mut self, key: Key) -> Result<EventState> {
        if key == self.key_config.filter {
            self.focus = Focus::Filter;
            return Ok(EventState::Consumed);
        }
        match key {
            key if matches!(self.focus, Focus::Filter) => return self.filter.event(key),
            key if matches!(self.focus, Focus::Table) => return self.table.event(key),
            _ => (),
        }
        Ok(EventState::NotConsumed)
    }
}
