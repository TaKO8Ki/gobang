use super::{
    compute_character_width, CompletionComponent, Component, EventState, MovableComponent,
    StatefulDrawableComponent,
};
use crate::components::command::CommandInfo;
use crate::components::utils::input::Input;
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
    pub input: Input,
    completion: CompletionComponent,
}

impl TableFilterComponent {
    pub fn new(key_config: KeyConfig) -> Self {
        Self {
            key_config: key_config.clone(),
            table: None,
            input: Input::new(),
            completion: CompletionComponent::new(key_config, "", false),
        }
    }

    pub fn reset(&mut self) {
        self.input.reset();
        self.table = None;
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

impl StatefulDrawableComponent for TableFilterComponent {
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
                if focused || !self.input.value.is_empty() {
                    self.input.value.iter().collect::<String>()
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
                    .map_or(String::new(), |table| {
                        format!("{} ", table.name.to_string())
                    })
                    .width() as u16)
                    .saturating_add(self.input.cursor_position),
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
                    .saturating_add(self.input.cursor_position)
                    .min(area.right().saturating_sub(2)),
                area.y + 1,
            )
        }
        Ok(())
    }
}

impl Component for TableFilterComponent {
    fn commands(&self, _out: &mut Vec<CommandInfo>) {}

    fn event(&mut self, key: Key) -> Result<EventState> {
        // apply comletion candidates
        if key == self.key_config.enter {
            return self.complete();
        }

        self.completion.selected_candidate();

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
            _ => self.completion.event(key),
        }
    }
}

#[cfg(test)]
mod test {
    use super::{KeyConfig, TableFilterComponent};

    #[test]
    fn test_complete() {
        let mut filter = TableFilterComponent::new(KeyConfig::default());
        filter.input.cursor_index = 2;
        filter.input.value = vec!['a', 'n', ' ', 'c', 'd', 'e', 'f', 'g'];
        filter.completion.update("an");
        assert!(filter.complete().is_ok());
        assert_eq!(
            filter.input.value,
            vec!['A', 'N', 'D', ' ', 'c', 'd', 'e', 'f', 'g']
        );
    }

    #[test]
    fn test_complete_end() {
        let mut filter = TableFilterComponent::new(KeyConfig::default());
        filter.input.cursor_index = 9;
        filter.input.value = vec!['a', 'b', ' ', 'c', 'd', 'e', 'f', ' ', 'i'];
        filter.completion.update('i');
        assert!(filter.complete().is_ok());
        assert_eq!(
            filter.input.value,
            vec!['a', 'b', ' ', 'c', 'd', 'e', 'f', ' ', 'I', 'N', ' ']
        );
    }

    #[test]
    fn test_complete_no_candidates() {
        let mut filter = TableFilterComponent::new(KeyConfig::default());
        filter.input.cursor_index = 2;
        filter.input.value = vec!['a', 'n', ' ', 'c', 'd', 'e', 'f', 'g'];
        filter.completion.update("foo");
        assert!(filter.complete().is_ok());
        assert_eq!(
            filter.input.value,
            vec!['a', 'n', ' ', 'c', 'd', 'e', 'f', 'g']
        );
    }
}
