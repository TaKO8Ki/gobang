use super::{
    compute_character_width, CompletionComponent, Component, DrawableComponent, EventState,
    MovableComponent,
};
use crate::components::command::CommandInfo;
use crate::config::KeyConfig;
use crate::event::Key;
use anyhow::Result;
use database_tree::Table;
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use unicode_width::UnicodeWidthStr;

pub struct TableFilterComponent {
    key_config: KeyConfig,
    pub table: Option<Table>,
    pub input: Vec<char>,
    input_idx: usize,
    input_cursor_position: u16,
    completion: CompletionComponent,
}

impl TableFilterComponent {
    pub fn new(key_config: KeyConfig) -> Self {
        Self {
            key_config: key_config.clone(),
            table: None,
            input: Vec::new(),
            input_idx: 0,
            input_cursor_position: 0,
            completion: CompletionComponent::new(key_config, ""),
        }
    }

    pub fn input_str(&self) -> String {
        self.input.iter().collect()
    }

    pub fn reset(&mut self) {
        self.table = None;
        self.input = Vec::new();
        self.input_idx = 0;
        self.input_cursor_position = 0;
    }

    fn update_completion(&mut self) {
        let input = &self
            .input
            .iter()
            .enumerate()
            .filter(|(i, _)| i <= &self.input_idx)
            .map(|(_, i)| i)
            .collect::<String>()
            .split(" ")
            .map(|i| i.to_string())
            .collect::<Vec<String>>();
        self.completion
            .update(input.last().unwrap_or(&String::new()));
    }
}

impl DrawableComponent for TableFilterComponent {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect, focused: bool) -> Result<()> {
        let query = Paragraph::new(Spans::from(vec![
            Span::styled(
                self.table
                    .as_ref()
                    .map_or("-".to_string(), |table| table.name.to_string()),
                Style::default().fg(Color::Blue),
            ),
            Span::from(format!(
                " {}",
                if focused || !self.input.is_empty() {
                    self.input.iter().collect::<String>()
                } else {
                    "Enter a SQL expression in WHERE clause to filter records".to_string()
                }
            )),
        ]))
        .style(if focused {
            Style::default()
        } else {
            Style::default().fg(Color::DarkGray)
        })
        .block(Block::default().borders(Borders::ALL));
        f.render_widget(query, area);

        if focused {
            self.completion.draw(
                f,
                area,
                false,
                (self
                    .table
                    .as_ref()
                    .map_or(String::new(), |table| table.name.to_string())
                    .width() as u16
                    + 2)
                .saturating_add(self.input_cursor_position),
                0,
            )?;
        };

        if focused {
            f.set_cursor(
                (area.x
                    + (1 + self
                        .table
                        .as_ref()
                        .map_or(String::new(), |table| table.name.to_string())
                        .width()
                        + 1) as u16)
                    .saturating_add(self.input_cursor_position),
                area.y + 1,
            )
        }
        Ok(())
    }
}

impl Component for TableFilterComponent {
    fn commands(&self, _out: &mut Vec<CommandInfo>) {}

    fn event(&mut self, key: Key) -> Result<EventState> {
        let input_str: String = self.input.iter().collect();

        // apply comletion candidates
        if key == self.key_config.enter {
            if let Some(candidate) = self.completion.string_to_be_completed() {
                let mut input = self.input.iter().collect::<String>();
                input.insert_str(self.input_idx, candidate.as_str());
                self.input = input.chars().collect();
                self.input_idx += candidate.len();
                self.input_cursor_position += candidate
                    .chars()
                    .map(|c| compute_character_width(c))
                    .sum::<u16>();
                self.update_completion();
                return Ok(EventState::Consumed);
            }
        }

        match key {
            Key::Char(c) => {
                self.input.insert(self.input_idx, c);
                self.input_idx += 1;
                self.input_cursor_position += compute_character_width(c);
                self.update_completion();

                Ok(EventState::Consumed)
            }
            Key::Delete | Key::Backspace => {
                if input_str.width() > 0 && !self.input.is_empty() && self.input_idx > 0 {
                    let last_c = self.input.remove(self.input_idx - 1);
                    self.input_idx -= 1;
                    self.input_cursor_position -= compute_character_width(last_c);
                    self.update_completion();
                }
                Ok(EventState::Consumed)
            }
            Key::Left => {
                if !self.input.is_empty() && self.input_idx > 0 {
                    self.input_idx -= 1;
                    self.input_cursor_position = self
                        .input_cursor_position
                        .saturating_sub(compute_character_width(self.input[self.input_idx]));
                }
                Ok(EventState::Consumed)
            }
            Key::Ctrl('a') => {
                if !self.input.is_empty() && self.input_idx > 0 {
                    self.input_idx = 0;
                    self.input_cursor_position = 0
                }
                Ok(EventState::Consumed)
            }
            Key::Right => {
                if self.input_idx < self.input.len() {
                    let next_c = self.input[self.input_idx];
                    self.input_idx += 1;
                    self.input_cursor_position += compute_character_width(next_c);
                }
                Ok(EventState::Consumed)
            }
            Key::Ctrl('e') => {
                if self.input_idx < self.input.len() {
                    self.input_idx = self.input.len();
                    self.input_cursor_position = self.input_str().width() as u16;
                }
                Ok(EventState::Consumed)
            }
            key => self.completion.event(key),
        }
    }
}
