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

#[cfg(test)]

mod test {
    use super::Input;
    use crate::components::compute_character_width;
    use crate::event::Key;
    use unicode_width::UnicodeWidthStr;

    #[test]
    fn test_adds_new_chars_for_char_key() {
        let mut input = Input::new();
        input.handle_key(Key::Char('a'));

        assert_eq!(input.value, vec!['a']);
        assert_eq!(input.cursor_index, 1);
        assert_eq!(input.cursor_position, compute_character_width('a'));
    }

    #[test]
    fn test_deletes_chars_for_backspace_and_delete_key() {
        let mut input = Input::new();
        input.value = vec!['a', 'b'];
        input.cursor_index = 2;
        input.cursor_position = input.value_str().width() as u16;

        input.handle_key(Key::Delete);
        input.handle_key(Key::Backspace);

        assert_eq!(input.value, Vec::<char>::new());
        assert_eq!(input.cursor_index, 0);
        assert_eq!(input.cursor_position, 0);
    }

    #[test]
    fn test_moves_cursor_left_for_left_key() {
        let mut input = Input::new();
        input.value = vec!['a'];
        input.cursor_index = 1;
        input.cursor_position = compute_character_width('a');

        let (matched_key, input_changed) = input.handle_key(Key::Left);

        assert_eq!(matched_key, Some(Key::Left));
        assert_eq!(input_changed, true);
        assert_eq!(input.value, vec!['a']);
        assert_eq!(input.cursor_index, 0);
        assert_eq!(input.cursor_position, 0);
    }

    #[test]
    fn test_moves_cursor_right_for_right_key() {
        let mut input = Input::new();
        input.value = vec!['a'];

        let (matched_key, input_changed) = input.handle_key(Key::Right);

        assert_eq!(matched_key, Some(Key::Right));
        assert_eq!(input_changed, true);
        assert_eq!(input.value, vec!['a']);
        assert_eq!(input.cursor_index, 1);
        assert_eq!(input.cursor_position, compute_character_width('a'));
    }

    #[test]
    fn test_jumps_to_beginning_for_ctrl_a() {
        let mut input = Input::new();
        input.value = vec!['a', 'b', 'c'];
        input.cursor_index = 3;
        input.cursor_position = input.value_str().width() as u16;

        let (matched_key, input_changed) = input.handle_key(Key::Ctrl('a'));

        assert_eq!(matched_key, Some(Key::Ctrl('a')));
        assert_eq!(input_changed, true);
        assert_eq!(input.value, vec!['a', 'b', 'c']);
        assert_eq!(input.cursor_index, 0);
        assert_eq!(input.cursor_position, 0);
    }

    #[test]
    fn test_jumps_to_end_for_ctrl_e() {
        let mut input = Input::new();
        input.value = vec!['a', 'b', 'c'];
        input.cursor_index = 0;
        input.cursor_position = 0;

        let (matched_key, input_changed) = input.handle_key(Key::Ctrl('e'));

        assert_eq!(matched_key, Some(Key::Ctrl('e')));
        assert_eq!(input_changed, true);
        assert_eq!(input.value, vec!['a', 'b', 'c']);
        assert_eq!(input.cursor_index, 3);
        assert_eq!(input.cursor_position, input.value_str().width() as u16);
    }
}
