use super::{Component, DrawableComponent, EventState};
use crate::event::Key;
use anyhow::Result;
use database_tree::Table;
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

pub struct TableStatusComponent {
    pub rows_count: u64,
    pub table: Option<Table>,
}

impl Default for TableStatusComponent {
    fn default() -> Self {
        Self {
            rows_count: 0,
            table: None,
        }
    }
}

impl TableStatusComponent {
    pub fn update(&mut self, count: u64, table: Table) {
        self.rows_count = count;
        self.table = Some(table);
    }

    pub fn status_str(&self) -> Vec<String> {
        if let Some(table) = self.table.as_ref() {
            return vec![
                format!(
                    "created: {}",
                    table
                        .create_time
                        .map(|time| time.to_string())
                        .unwrap_or_default()
                ),
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
                format!("rows: {}", self.rows_count),
            ];
        }
        Vec::new()
    }
}

impl DrawableComponent for TableStatusComponent {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect, focused: bool) -> Result<()> {
        let table_status: Vec<ListItem> = self
            .status_str()
            .iter()
            .map(|i| {
                ListItem::new(vec![Spans::from(Span::raw(i.to_string()))]).style(Style::default())
            })
            .collect();
        let tasks = List::new(table_status).block(Block::default().borders(Borders::ALL).style(
            if focused {
                Style::default()
            } else {
                Style::default().fg(Color::DarkGray)
            },
        ));
        f.render_widget(tasks, area);
        Ok(())
    }
}

impl Component for TableStatusComponent {
    fn event(&mut self, _key: Key) -> Result<EventState> {
        Ok(EventState::NotConsumed)
    }
}
