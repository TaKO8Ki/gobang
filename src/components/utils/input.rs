use super::{is_nonalphanumeric, is_whitespace};
use crate::components::compute_character_width;
use crate::event::Key;
use std::ops::Range;
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

    pub fn value_width(&self) -> u16 {
        self.value_str().width() as u16
    }

    fn width_for(&self, chars: &[char]) -> u16 {
        chars.iter().collect::<String>().width() as u16
    }

    pub fn reset(&mut self) {
        self.value = Vec::new();
        self.cursor_index = 0;
        self.cursor_position = 0;
    }

    fn cannot_go_left(&self) -> bool {
        self.value.is_empty() || self.cursor_index == 0 || self.value_width() == 0
    }

    fn cannot_go_right(&self) -> bool {
        self.cursor_index == self.value.len()
    }

    fn find_index_for_char_of_kind(
        &self,
        range: Range<usize>,
        is_char_of_kind: &dyn Fn(char) -> bool,
    ) -> Option<usize> {
        let mut result = None;

        for i in range {
            if is_char_of_kind(self.value[i]) {
                result = Some(i);
                break;
            }
        }

        return result;
    }

    fn cursor_index_backwards_until(&self, is_char_of_kind: &dyn Fn(char) -> bool) -> usize {
        let range = 0..self.cursor_index - 1;

        match self.find_index_for_char_of_kind(range, is_char_of_kind) {
            Some(index) => index + 1,
            None => 0,
        }
    }

    fn cursor_index_forwards_until(&self, is_char_of_kind: &dyn Fn(char) -> bool) -> usize {
        let range = self.cursor_index + 1..self.value.len();

        match self.find_index_for_char_of_kind(range, is_char_of_kind) {
            Some(index) => index,
            None => self.value.len(),
        }
    }

    fn delete_left_until(&mut self, new_cursor_index: usize) {
        let mut tail = self.value.to_vec().drain(self.cursor_index..).collect();

        self.cursor_index = new_cursor_index;
        self.value.truncate(new_cursor_index);
        self.cursor_position = self.value_width();
        self.value.append(&mut tail);
    }

    fn delete_right_until(&mut self, index: usize) {
        let mut tail = self.value.to_vec().drain(index..).collect();

        self.value.truncate(self.cursor_index);
        self.value.append(&mut tail);
    }

    pub fn handle_key(&mut self, key: Key) -> (Option<Key>, bool) {
        match key {
            Key::Char(c) => {
                self.value.insert(self.cursor_index, c);
                self.cursor_index += 1;
                self.cursor_position += compute_character_width(c);

                return (Some(key), true);
            }
            Key::Delete | Key::Backspace => {
                if self.cannot_go_left() {
                    return (Some(key), false);
                }

                let last_c = self.value.remove(self.cursor_index - 1);
                self.cursor_index -= 1;
                self.cursor_position -= compute_character_width(last_c);
                return (Some(key), true);
            }
            Key::Right | Key::Ctrl('f') => {
                if self.cannot_go_right() {
                    return (Some(key), false);
                }

                let next_c = self.value[self.cursor_index];
                self.cursor_index += 1;
                self.cursor_position += compute_character_width(next_c);
                return (Some(key), true);
            }
            Key::Ctrl('e') => {
                if self.cannot_go_right() {
                    return (Some(key), false);
                }

                self.cursor_index = self.value.len();
                self.cursor_position = self.value_width();
                return (Some(key), true);
            }
            Key::Alt('f') => {
                if self.cannot_go_right() {
                    return (Some(key), false);
                }

                let new_cursor_index = self.cursor_index_forwards_until(&is_nonalphanumeric);
                self.cursor_index = new_cursor_index;
                self.cursor_position = self.width_for(&self.value[0..new_cursor_index]);
                return (Some(key), true);
            }
            Key::Alt('d') => {
                if self.cannot_go_right() {
                    return (Some(key), false);
                }

                let index = self.cursor_index_forwards_until(&is_nonalphanumeric);
                self.delete_right_until(index);
                return (Some(key), true);
            }
            Key::Left | Key::Ctrl('b') => {
                if self.cannot_go_left() {
                    return (Some(key), false);
                }

                self.cursor_index -= 1;
                self.cursor_position = self
                    .cursor_position
                    .saturating_sub(compute_character_width(self.value[self.cursor_index]));
                return (Some(key), true);
            }
            Key::Ctrl('a') => {
                if self.cannot_go_left() {
                    return (Some(key), false);
                }

                self.cursor_index = 0;
                self.cursor_position = 0;
                return (Some(key), true);
            }
            Key::Alt('b') => {
                if self.cannot_go_left() {
                    return (Some(key), false);
                }

                let new_cursor_index = self.cursor_index_backwards_until(&is_nonalphanumeric);
                self.cursor_index = new_cursor_index;
                self.cursor_position = self.width_for(&self.value[0..new_cursor_index]);
                return (Some(key), true);
            }
            Key::Ctrl('w') => {
                if self.cannot_go_left() {
                    return (Some(key), false);
                }

                let new_cursor_index = self.cursor_index_backwards_until(&is_whitespace);
                self.delete_left_until(new_cursor_index);

                return (Some(key), true);
            }
            Key::AltBackspace => {
                if self.cannot_go_left() {
                    return (Some(key), false);
                }

                let new_cursor_index = self.cursor_index_backwards_until(&is_nonalphanumeric);
                self.delete_left_until(new_cursor_index);

                return (Some(key), true);
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
        input.cursor_position = input.value_width();

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
        input.cursor_position = input.value_width();

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
        assert_eq!(input.cursor_position, input.value_width());
    }

    #[test]
    fn test_deletes_word_for_ctrl_w() {
        let mut input = Input::new();
        input.value = vec!['a', ' ', 'c', 'd'];
        input.cursor_index = 3;
        input.cursor_position = input.value_width();

        let (matched_key, input_changed) = input.handle_key(Key::Ctrl('w'));

        assert_eq!(matched_key, Some(Key::Ctrl('w')));
        assert_eq!(input_changed, true);
        assert_eq!(input.value, vec!['a', ' ', 'd']);
        assert_eq!(input.cursor_index, 2);
    }

    #[test]
    fn test_deletes_backwards_til_nonalphanumeric_for_alt_backspace() {
        let mut input = Input::new();
        input.value = vec!['a', '-', 'c', 'd'];
        input.cursor_index = 3;
        input.cursor_position = input.value_width();

        let (matched_key, input_changed) = input.handle_key(Key::AltBackspace);

        assert_eq!(matched_key, Some(Key::AltBackspace));
        assert_eq!(input_changed, true);
        assert_eq!(input.value, vec!['a', '-', 'd']);
        assert_eq!(input.cursor_index, 2);
    }

    #[test]
    fn test_deletes_forwards_til_nonalphanumeric_for_alt_d() {
        let mut input = Input::new();
        input.value = vec!['a', 'b', '-', 'd'];
        input.cursor_index = 1;
        input.cursor_position = input.value_width();

        let (matched_key, input_changed) = input.handle_key(Key::Alt('d'));

        assert_eq!(matched_key, Some(Key::Alt('d')));
        assert_eq!(input_changed, true);
        assert_eq!(input.value, vec!['a', '-', 'd']);
        assert_eq!(input.cursor_index, 1);
    }

    #[test]
    fn test_moves_backwards_til_nonalphanumeric_for_alt_b() {
        let mut input = Input::new();
        input.value = vec!['a', '-', 'c', 'd'];
        input.cursor_index = 3;
        input.cursor_position = input.value_width();

        let (matched_key, input_changed) = input.handle_key(Key::Alt('b'));

        assert_eq!(matched_key, Some(Key::Alt('b')));
        assert_eq!(input_changed, true);
        assert_eq!(input.value, vec!['a', '-', 'c', 'd']);
        assert_eq!(input.cursor_index, 2);
    }

    #[test]
    fn test_moves_forwards_til_nonalphanumeric_for_alt_f() {
        let mut input = Input::new();
        input.value = vec!['a', 'b', '-', 'c'];
        input.cursor_index = 1;
        input.cursor_position = input.value_width();

        let (matched_key, input_changed) = input.handle_key(Key::Alt('f'));

        assert_eq!(matched_key, Some(Key::Alt('f')));
        assert_eq!(input_changed, true);
        assert_eq!(input.value, vec!['a', 'b', '-', 'c']);
        assert_eq!(input.cursor_index, 2);
    }
}
