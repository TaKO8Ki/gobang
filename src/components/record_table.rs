use super::{Component, DrawableComponent, EventState};
use crate::components::{TableComponent, TableFilterComponent};
use crate::event::Key;
use anyhow::Result;
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
}

impl Default for RecordTableComponent {
    fn default() -> Self {
        Self {
            filter: TableFilterComponent::default(),
            table: TableComponent::default(),
            focus: Focus::Table,
        }
    }
}

impl RecordTableComponent {
    pub fn new(rows: Vec<Vec<String>>, headers: Vec<String>) -> Self {
        Self {
            table: TableComponent::new(rows, headers),
            ..Self::default()
        }
    }

    pub fn update(&mut self, rows: Vec<Vec<String>>, headers: Vec<String>) {
        self.table.rows = rows;
        self.table.headers = headers;
        if !self.table.rows.is_empty() {
            self.table.state.select(None);
            self.table.state.select(Some(0));
        }
    }

    pub fn reset(&mut self) {
        self.table = TableComponent::default();
        if !self.table.rows.is_empty() {
            self.table.state.select(None);
            self.table.state.select(Some(0))
        }
        self.filter = TableFilterComponent::default();
    }

    pub fn len(&self) -> usize {
        self.table.rows.len()
    }

    pub fn set_table(&mut self, table: String) {
        self.filter.table = Some(table)
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

        self.filter
            .draw(f, layout[0], focused && matches!(self.focus, Focus::Filter))?;

        self.table
            .draw(f, layout[1], focused && matches!(self.focus, Focus::Table))?;
        Ok(())
    }
}

impl Component for RecordTableComponent {
    fn event(&mut self, key: Key) -> Result<EventState> {
        match key {
            Key::Char('/') => {
                self.focus = Focus::Filter;
                return Ok(EventState::Consumed);
            }
            key if matches!(self.focus, Focus::Filter) => return self.filter.event(key),
            key if matches!(self.focus, Focus::Table) => return self.table.event(key),
            _ => (),
        }
        Ok(EventState::NotConsumed)
    }
}
