use crate::components::compute_character_width;
use crate::event::Key;
use unicode_width::UnicodeWidthStr;

pub struct Input {
    pub value: Vec<char>,
    pub cursor_position: u16,
    pub cursor_index: usize,
}

impl Input {
    pub fn new() -> Self {
        Self {
            value: Vec::new(),
            cursor_index: 0,
            cursor_position: 0,
        }
    }

    pub fn value_str(&self) -> String {
        self.value.iter().collect()
    }

    pub fn reset(&mut self) {
        self.value = Vec::new();
        self.cursor_index = 0;
        self.cursor_position = 0;
    }

    pub fn handle_key(&mut self, key: Key) -> (Option<Key>, bool) {
        let value_str: String = self.value.iter().collect();

        match key {
            Key::Char(c) => {
                self.value.insert(self.cursor_index, c);
                self.cursor_index += 1;
                self.cursor_position += compute_character_width(c);

                return (Some(key), true);
            }
            Key::Delete | Key::Backspace => {
                if value_str.width() > 0 && !self.value.is_empty() && self.cursor_index > 0 {
                    let last_c = self.value.remove(self.cursor_index - 1);
                    self.cursor_index -= 1;
                    self.cursor_position -= compute_character_width(last_c);
                    return (Some(key), true);
                }
                return (Some(key), false);
            }
            Key::Left => {
                if !self.value.is_empty() && self.cursor_index > 0 {
                    self.cursor_index -= 1;
                    self.cursor_position = self
                        .cursor_position
                        .saturating_sub(compute_character_width(self.value[self.cursor_index]));
                    return (Some(key), true);
                }
                return (Some(key), false);
            }
            Key::Right => {
                if self.cursor_index < self.value.len() {
                    let next_c = self.value[self.cursor_index];
                    self.cursor_index += 1;
                    self.cursor_position += compute_character_width(next_c);
                    return (Some(key), true);
                }
                return (Some(key), false);
            }
            Key::Ctrl('a') => {
                if !self.value.is_empty() && self.cursor_index > 0 {
                    self.cursor_index = 0;
                    self.cursor_position = 0;
                    return (Some(key), true);
                }
                return (Some(key), false);
            }
            Key::Ctrl('e') => {
                if self.cursor_index < self.value.len() {
                    self.cursor_index = self.value.len();
                    self.cursor_position = self.value_str().width() as u16;
                    return (Some(key), true);
                }
                return (Some(key), false);
            }
            _ => (None, false),
        }
    }
}
