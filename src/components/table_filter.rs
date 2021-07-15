use super::{Component, DrawableComponent};
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
    pub input: String,
    pub input_cursor_x: u16,
}

impl Default for TableFilterComponent {
    fn default() -> Self {
        Self {
            table: None,
            input: String::new(),
            input_cursor_x: 0,
        }
    }
}

impl TableFilterComponent {
    pub fn increment_input_cursor_x(&mut self) {
        if self.input_cursor_x > 0 {
            self.input_cursor_x -= 1;
        }
    }

    pub fn decrement_input_cursor_x(&mut self) {
        if self.input_cursor_x < self.input.width() as u16 {
            self.input_cursor_x += 1;
        }
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
                    self.input.as_ref()
                } else {
                    "Enter a SQL expression in WHERE clause"
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
                    + self.input.width() as u16
                    + 1
                    + self
                        .table
                        .as_ref()
                        .map_or(String::new(), |table| table.to_string())
                        .width() as u16
                    + 1)
                .saturating_sub(self.input_cursor_x),
                area.y + 1,
            )
        }
        Ok(())
    }
}

impl Component for TableFilterComponent {
    fn event(&mut self, key: Key) -> Result<()> {
        match key {
            Key::Char(c) => self.input.push(c),
            Key::Delete | Key::Backspace => {
                if self.input.width() > 0 {
                    if self.input_cursor_x == 0 {
                        self.input.pop();
                        return Ok(());
                    }
                    if self.input.width() - self.input_cursor_x as usize > 0 {
                        self.input
                            .remove(self.input.width() - self.input_cursor_x as usize);
                    }
                }
            }
            Key::Left => self.decrement_input_cursor_x(),
            Key::Right => self.increment_input_cursor_x(),
            _ => (),
        }
        Ok(())
    }
}
