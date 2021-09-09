use super::{
    compute_character_width, Component, EventState, StatefulDrawableComponent, TableComponent,
};
use crate::components::command::CommandInfo;
use crate::config::KeyConfig;
use crate::database::{ExecuteResult, Pool};
use crate::event::Key;
use crate::ui::syntax_text::SyntaxText;
use anyhow::Result;
use async_trait::async_trait;
use database_tree::Table;
use sqlx::query::Query;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

struct QueryResult {
    updated_rows: u64,
    query: String,
}

pub enum Focus {
    Editor,
    Table,
}

pub struct SqlEditorComponent {
    input: Vec<char>,
    table: TableComponent,
    query_result: Option<QueryResult>,
    key_config: KeyConfig,
    focus: Focus,
}

impl SqlEditorComponent {
    pub fn new(key_config: KeyConfig) -> Self {
        Self {
            input: Vec::new(),
            table: TableComponent::new(key_config.clone()),
            focus: Focus::Editor,
            query_result: None,
            key_config,
        }
    }
}

impl StatefulDrawableComponent for SqlEditorComponent {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect, focused: bool) -> Result<()> {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(if matches!(self.focus, Focus::Table) {
                vec![Constraint::Length(7), Constraint::Min(1)]
            } else {
                vec![Constraint::Percentage(50), Constraint::Percentage(50)]
            })
            .split(area);

        let editor = Paragraph::new(self.input.iter().collect::<String>())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("SQL Editor")
                    .style(if focused && matches!(self.focus, Focus::Editor) {
                        Style::default()
                    } else {
                        Style::default().fg(Color::DarkGray)
                    }),
            )
            .wrap(Wrap { trim: true });
        f.render_widget(editor, layout[0]);

        self.table
            .draw(f, layout[1], matches!(self.focus, Focus::Table))?;
        Ok(())
    }
}

#[async_trait]
impl Component for SqlEditorComponent {
    fn commands(&self, _out: &mut Vec<CommandInfo>) {}

    fn event(&mut self, key: Key) -> Result<EventState> {
        match key {
            Key::Char(c) => {
                self.input.push(c);

                return Ok(EventState::Consumed);
            }
            _ => (),
        }

        Ok(EventState::NotConsumed)
    }

    async fn async_event(&mut self, key: Key, pool: &Box<dyn Pool>) -> Result<EventState> {
        if key == self.key_config.enter {
            let query = self.input.iter().collect();
            let result = pool.execute(&query).await?;
            match result {
                ExecuteResult::Read {
                    headers,
                    rows,
                    database,
                    table,
                } => {
                    self.table.update(rows, headers, database, table);
                    self.focus = Focus::Table
                }
                ExecuteResult::Write { updated_rows } => {
                    self.query_result = Some(QueryResult {
                        updated_rows,
                        query: query.to_string(),
                    })
                }
            }
            return Ok(EventState::Consumed);
        }

        Ok(EventState::NotConsumed)
    }
}
