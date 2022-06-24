use super::{Component, DrawableComponent, EventState};
use crate::components::command::CommandInfo;
use crate::components::utils::input::Input;
use crate::event::Key;
use anyhow::Result;
use database_tree::Table;
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Style},
    text::Spans,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct DatabaseFilterComponent {
    pub table: Option<Table>,
    pub input: Input,
}

impl DatabaseFilterComponent {
    pub fn new() -> Self {
        Self {
            table: None,
            input: Input::new(),
        }
    }

    pub fn reset(&mut self) {
        self.table = None;
        self.input.reset();
    }
}

impl DrawableComponent for DatabaseFilterComponent {
    fn draw<B: Backend>(&self, f: &mut Frame<B>, area: Rect, focused: bool) -> Result<()> {
        let query = Paragraph::new(Spans::from(format!(
            "{:w$}",
            if self.input.value.is_empty() && !focused {
                "Filter tables".to_string()
            } else {
                self.input.value_str()
            },
            w = area.width as usize
        )))
        .style(if focused {
            Style::default()
        } else {
            Style::default().fg(Color::DarkGray)
        })
        .block(Block::default().borders(Borders::BOTTOM));
        f.render_widget(query, area);

        if focused {
            f.set_cursor(
                (area.x + self.input.cursor_position).min(area.right().saturating_sub(1)),
                area.y,
            )
        }
        Ok(())
    }
}

impl Component for DatabaseFilterComponent {
    fn commands(&self, _out: &mut Vec<CommandInfo>) {}

    fn event(&mut self, key: Key) -> Result<EventState> {
        match self.input.handle_key(key) {
            (Some(_), _) => Ok(EventState::Consumed),
            _ => Ok(EventState::NotConsumed),
        }
    }
}
