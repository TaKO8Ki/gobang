use super::{Component, DrawableComponent};
use crate::event::Key;
use anyhow::Result;
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use unicode_width::UnicodeWidthStr;

pub struct QueryComponent {
    pub input: String,
    pub input_cursor_x: u16,
}

impl Default for QueryComponent {
    fn default() -> Self {
        Self {
            input: String::new(),
            input_cursor_x: 0,
        }
    }
}

impl QueryComponent {
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

impl DrawableComponent for QueryComponent {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect, focused: bool) -> Result<()> {
        let query = Paragraph::new(self.input.as_ref())
            .style(if focused {
                Style::default()
            } else {
                Style::default().fg(Color::DarkGray)
            })
            .block(Block::default().borders(Borders::ALL).title("Query"));
        f.render_widget(query, area);
        if focused {
            f.set_cursor(
                area.x + self.input.width() as u16 + 1 - self.input_cursor_x,
                area.y + 1,
            )
        }
        Ok(())
    }
}

impl Component for QueryComponent {
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
