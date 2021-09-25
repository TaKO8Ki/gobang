use super::{
    compute_character_width, CompletionComponent, Component, EventState, MovableComponent,
    StatefulDrawableComponent, TableComponent,
};
use crate::components::command::CommandInfo;
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
    query: String,
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
    input: Vec<char>,
    input_cursor_position_x: u16,
    input_idx: usize,
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
            input: Vec::new(),
            input_idx: 0,
            input_cursor_position_x: 0,
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
            .iter()
            .enumerate()
            .filter(|(i, _)| i < &self.input_idx)
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
                .iter()
                .enumerate()
                .filter(|(i, _)| i < &self.input_idx.saturating_sub(self.completion.word().len()))
                .map(|(_, c)| c.to_string())
                .collect::<Vec<String>>();
            let last = self
                .input
                .iter()
                .enumerate()
                .filter(|(i, _)| i >= &self.input_idx)
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

            self.input = input.join("").chars().collect();
            self.input_idx += &middle.len();
            if is_last_word {
                self.input_idx += 1;
            }
            self.input_idx -= self.completion.word().len();
            self.input_cursor_position_x += middle
                .join("")
                .chars()
                .map(compute_character_width)
                .sum::<u16>();
            if is_last_word {
                self.input_cursor_position_x += " ".to_string().width() as u16
            }
            self.input_cursor_position_x -= self
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
                vec![Constraint::Percentage(50), Constraint::Percentage(50)]
            })
            .split(area);

        // let editor = Paragraph::new(self.input.iter().collect::<String>())
        //     .block(
        //         Block::default()
        //             .borders(Borders::ALL)
        //             .title("SQL Editor")
        //             .style(if focused && matches!(self.focus, Focus::Editor) {
        //                 Style::default()
        //             } else {
        //                 Style::default().fg(Color::DarkGray)
        //             }),
        //     )
        //     .wrap(Wrap { trim: true });
        // f.render_widget(editor, layout[0]);

        let content = StatefulParagraph::new(self.input.iter().collect::<String>())
            .wrap(Wrap { trim: false })
            .block(Block::default().borders(Borders::ALL));

        f.render_stateful_widget(content, area, &mut self.paragraph_state);

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
                    .saturating_add(
                        self.input_cursor_position_x % layout[0].width.saturating_sub(2),
                    )
                    .min(area.right().saturating_sub(2)),
                (layout[0].y
                    + 1
                    + self.input_cursor_position_x / layout[0].width.saturating_sub(2))
                .min(layout[0].bottom()),
            )
        }

        if focused && matches!(self.focus, Focus::Editor) {
            self.completion.draw(
                f,
                area,
                false,
                self.input_cursor_position_x % layout[0].width.saturating_sub(2) + 1,
                self.input_cursor_position_x / layout[0].width.saturating_sub(2),
            )?;
        };
        Ok(())
    }
}

#[async_trait]
impl Component for SqlEditorComponent {
    fn commands(&self, _out: &mut Vec<CommandInfo>) {}

    fn event(&mut self, key: Key) -> Result<EventState> {
        let input_str: String = self.input.iter().collect();

        if key == self.key_config.focus_above && matches!(self.focus, Focus::Table) {
            self.focus = Focus::Editor
        } else if key == self.key_config.enter {
            return self.complete();
        }

        match key {
            Key::Char(c) if matches!(self.focus, Focus::Editor) => {
                self.input.insert(self.input_idx, c);
                self.input_idx += 1;
                self.input_cursor_position_x += compute_character_width(c);
                self.update_completion();

                return Ok(EventState::Consumed);
            }
            Key::Esc if matches!(self.focus, Focus::Editor) => self.focus = Focus::Table,
            Key::Delete | Key::Backspace if matches!(self.focus, Focus::Editor) => {
                if input_str.width() > 0 && !self.input.is_empty() && self.input_idx > 0 {
                    let last_c = self.input.remove(self.input_idx - 1);
                    self.input_idx -= 1;
                    self.input_cursor_position_x -= compute_character_width(last_c);
                    self.completion.update("");
                }

                return Ok(EventState::Consumed);
            }
            Key::Left if matches!(self.focus, Focus::Editor) => {
                if !self.input.is_empty() && self.input_idx > 0 {
                    self.input_idx -= 1;
                    self.input_cursor_position_x = self
                        .input_cursor_position_x
                        .saturating_sub(compute_character_width(self.input[self.input_idx]));
                    self.completion.update("");
                }
                return Ok(EventState::Consumed);
            }
            Key::Right if matches!(self.focus, Focus::Editor) => {
                if self.input_idx < self.input.len() {
                    let next_c = self.input[self.input_idx];
                    self.input_idx += 1;
                    self.input_cursor_position_x += compute_character_width(next_c);
                    self.completion.update("");
                }
                return Ok(EventState::Consumed);
            }
            key if matches!(self.focus, Focus::Table) => return self.table.event(key),
            _ => (),
        }
        return Ok(EventState::NotConsumed);
    }

    async fn async_event(&mut self, key: Key, pool: &Box<dyn Pool>) -> Result<EventState> {
        if key == self.key_config.run && matches!(self.focus, Focus::Editor) {
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
                    self.focus = Focus::Table;
                    self.query_result = None;
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
