use super::{compute_character_width, Component, DrawableComponent, EventState};
use crate::components::command::CommandInfo;
use crate::event::Key;
use anyhow::Result;
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
    pub table: Option<String>,
    pub input: Vec<char>,
    input_idx: usize,
    input_cursor_position: u16,
}

impl Default for TableFilterComponent {
    fn default() -> Self {
        Self {
            table: None,
            input: Vec::new(),
            input_idx: 0,
            input_cursor_position: 0,
        }
    }
}

impl TableFilterComponent {
    pub fn input_str(&self) -> String {
        self.input.iter().collect()
    }
}

impl DrawableComponent for TableFilterComponent {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect, focused: bool) -> Result<()> {
        let query = Paragraph::new(Spans::from(vec![
            Span::styled(
                self.table
                    .as_ref()
                    .map_or("-".to_string(), |table| table.to_string()),
                Style::default().fg(Color::Blue),
            ),
            Span::from(format!(
                " {}",
                if focused || !self.input.is_empty() {
                    self.input.iter().collect::<String>()
                } else {
                    "Enter a SQL expression in WHERE clause".to_string()
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
            f.set_cursor(
                (area.x
                    + (1 + self
                        .table
                        .as_ref()
                        .map_or(String::new(), |table| table.to_string())
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
    fn commands(&self, out: &mut Vec<CommandInfo>) {}

    fn event(&mut self, key: Key) -> Result<EventState> {
        let input_str: String = self.input.iter().collect();
        match key {
            Key::Char(c) => {
                self.input.insert(self.input_idx, c);
                self.input_idx += 1;
                self.input_cursor_position += compute_character_width(c);

                return Ok(EventState::Consumed);
            }
            Key::Delete | Key::Backspace => {
                if input_str.width() > 0 && !self.input.is_empty() && self.input_idx > 0 {
                    let last_c = self.input.remove(self.input_idx - 1);
                    self.input_idx -= 1;
                    self.input_cursor_position -= compute_character_width(last_c);
                }
                return Ok(EventState::Consumed);
            }
            Key::Left => {
                if !self.input.is_empty() && self.input_idx > 0 {
                    self.input_idx -= 1;
                    self.input_cursor_position = self
                        .input_cursor_position
                        .saturating_sub(compute_character_width(self.input[self.input_idx]));
                }
                return Ok(EventState::Consumed);
            }
            Key::Ctrl('a') => {
                if !self.input.is_empty() && self.input_idx > 0 {
                    self.input_idx = 0;
                    self.input_cursor_position = 0
                }
                return Ok(EventState::Consumed);
            }
            Key::Right => {
                if self.input_idx < self.input.len() {
                    let next_c = self.input[self.input_idx];
                    self.input_idx += 1;
                    self.input_cursor_position += compute_character_width(next_c);
                }
                return Ok(EventState::Consumed);
            }
            Key::Ctrl('e') => {
                if self.input_idx < self.input.len() {
                    self.input_idx = self.input.len();
                    self.input_cursor_position = self.input_str().width() as u16;
                }
                return Ok(EventState::Consumed);
            }
            _ => (),
        }
        Ok(EventState::NotConsumed)
    }
}
