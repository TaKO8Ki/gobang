use super::{
    compute_character_width, CompletionComponent, Component, EventState, MovableComponent,
    StatefulDrawableComponent, TableComponent,
};
use crate::components::command::CommandInfo;
use crate::components::utils::input::Input;
use crate::config::KeyConfig;
use crate::database::{ExecuteResult, Pool};
use crate::event::Key;
use crate::ui::stateful_paragraph::{ParagraphState, StatefulParagraph};
use anyhow::Result;
use async_trait::async_trait;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use unicode_width::UnicodeWidthStr;

struct QueryResult {
    updated_rows: u64,
}

impl QueryResult {
    fn result_str(&self) -> String {
        format!("Query OK, {} row affected", self.updated_rows)
    }
}

pub enum Focus {
    Editor,
    Table,
}

pub struct SqlEditorComponent {
    input: Input,
    table: TableComponent,
    query_result: Option<QueryResult>,
    completion: CompletionComponent,
    key_config: KeyConfig,
    paragraph_state: ParagraphState,
    focus: Focus,
}

impl SqlEditorComponent {
    pub fn new(key_config: KeyConfig) -> Self {
        Self {
            input: Input::new(),
            table: TableComponent::new(key_config.clone()),
            completion: CompletionComponent::new(key_config.clone(), "", true),
            focus: Focus::Editor,
            paragraph_state: ParagraphState::default(),
            query_result: None,
            key_config,
        }
    }

    fn update_completion(&mut self) {
        let input = &self
            .input
            .value
            .iter()
            .enumerate()
            .filter(|(i, _)| i < &self.input.cursor_index)
            .map(|(_, i)| i)
            .collect::<String>()
            .split(' ')
            .map(|i| i.to_string())
            .collect::<Vec<String>>();
        self.completion
            .update(input.last().unwrap_or(&String::new()));
    }

    fn complete(&mut self) -> anyhow::Result<EventState> {
        if let Some(candidate) = self.completion.selected_candidate() {
            let mut input = Vec::new();
            let first = self
                .input
                .value
                .iter()
                .enumerate()
                .filter(|(i, _)| {
                    i < &self
                        .input
                        .cursor_index
                        .saturating_sub(self.completion.word().len())
                })
                .map(|(_, c)| c.to_string())
                .collect::<Vec<String>>();
            let last = self
                .input
                .value
                .iter()
                .enumerate()
                .filter(|(i, _)| i >= &self.input.cursor_index)
                .map(|(_, c)| c.to_string())
                .collect::<Vec<String>>();

            let is_last_word = last.first().map_or(false, |c| c == &" ".to_string());

            let middle = if is_last_word {
                candidate
                    .chars()
                    .map(|c| c.to_string())
                    .collect::<Vec<String>>()
            } else {
                let mut c = candidate
                    .chars()
                    .map(|c| c.to_string())
                    .collect::<Vec<String>>();
                c.push(" ".to_string());
                c
            };

            input.extend(first);
            input.extend(middle.clone());
            input.extend(last);

            self.input.value = input.join("").chars().collect();
            self.input.cursor_index += &middle.len();
            if is_last_word {
                self.input.cursor_index += 1;
            }
            self.input.cursor_index -= self.completion.word().len();
            self.input.cursor_position += middle
                .join("")
                .chars()
                .map(compute_character_width)
                .sum::<u16>();
            if is_last_word {
                self.input.cursor_position += " ".to_string().width() as u16
            }
            self.input.cursor_position -= self
                .completion
                .word()
                .chars()
                .map(compute_character_width)
                .sum::<u16>();
            self.update_completion();
            return Ok(EventState::Consumed);
        }
        Ok(EventState::NotConsumed)
    }
}

impl StatefulDrawableComponent for SqlEditorComponent {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect, focused: bool) -> Result<()> {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(if matches!(self.focus, Focus::Table) {
                vec![Constraint::Length(7), Constraint::Min(1)]
            } else {
                vec![Constraint::Percentage(50), Constraint::Min(1)]
            })
            .split(area);

        let editor = StatefulParagraph::new(self.input.value.iter().collect::<String>())
            .wrap(Wrap { trim: true })
            .block(Block::default().borders(Borders::ALL));

        f.render_stateful_widget(editor, layout[0], &mut self.paragraph_state);

        if let Some(result) = self.query_result.as_ref() {
            let result = Paragraph::new(result.result_str())
                .block(Block::default().borders(Borders::ALL).style(
                    if focused && matches!(self.focus, Focus::Editor) {
                        Style::default()
                    } else {
                        Style::default().fg(Color::DarkGray)
                    },
                ))
                .wrap(Wrap { trim: true });
            f.render_widget(result, layout[1]);
        } else {
            self.table
                .draw(f, layout[1], focused && matches!(self.focus, Focus::Table))?;
        }

        if focused && matches!(self.focus, Focus::Editor) {
            f.set_cursor(
                (layout[0].x + 1)
                    .saturating_add(self.input.cursor_position % layout[0].width.saturating_sub(2))
                    .min(area.right().saturating_sub(2)),
                (layout[0].y + 1 + self.input.cursor_position / layout[0].width.saturating_sub(2))
                    .min(layout[0].bottom()),
            )
        }

        if focused && matches!(self.focus, Focus::Editor) {
            self.completion.draw(
                f,
                area,
                false,
                self.input.cursor_position % layout[0].width.saturating_sub(2) + 1,
                self.input.cursor_position / layout[0].width.saturating_sub(2),
            )?;
        };
        Ok(())
    }
}

#[async_trait]
impl Component for SqlEditorComponent {
    fn commands(&self, _out: &mut Vec<CommandInfo>) {}

    fn event(&mut self, key: Key) -> Result<EventState> {
        if key == self.key_config.focus_above && matches!(self.focus, Focus::Table) {
            self.focus = Focus::Editor
        } else if key == self.key_config.enter {
            return self.complete();
        }

        if matches!(self.focus, Focus::Table) {
            return self.table.event(key);
        }

        if !matches!(self.focus, Focus::Editor) {
            return Ok(EventState::NotConsumed);
        }

        if key == Key::Esc {
            self.focus = Focus::Table;
            return Ok(EventState::Consumed);
        } else {
            match self.input.handle_key(key) {
                (Some(matched_key), input_updated) => match matched_key {
                    Key::Char(_) => {
                        self.update_completion();
                        return Ok(EventState::Consumed);
                    }
                    Key::Ctrl(_) => {
                        return Ok(EventState::Consumed);
                    }
                    _ => {
                        if input_updated {
                            self.completion.update("");
                        }
                        return Ok(EventState::Consumed);
                    }
                },
                _ => return Ok(EventState::NotConsumed),
            }
        }
    }

    async fn async_event(&mut self, key: Key, pool: &Box<dyn Pool>) -> Result<EventState> {
        if key == self.key_config.enter && matches!(self.focus, Focus::Editor) {
            let query = self.input.value.iter().collect();
            let result = pool.execute(&query).await?;
            match result {
                ExecuteResult::Read {
                    headers,
                    rows,
                    database,
                    table,
                } => {
                    self.table.update(rows, headers, database, table);
                    self.focus = Focus::Table;
                    self.query_result = None;
                }
                ExecuteResult::Write { updated_rows } => {
                    self.query_result = Some(QueryResult { updated_rows })
                }
            }
            return Ok(EventState::Consumed);
        }

        Ok(EventState::NotConsumed)
    }
}
