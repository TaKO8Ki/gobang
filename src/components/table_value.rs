use super::{Component, DrawableComponent, EventState};
use crate::event::Key;
use anyhow::Result;
use tui::{
    backend::Backend,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub struct TableValueComponent {
    pub value: String,
}

impl TableValueComponent {
    pub fn new(value: String) -> Self {
        Self { value }
    }
}

impl DrawableComponent for TableValueComponent {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect, focused: bool) -> Result<()> {
        let paragraph = Paragraph::new(self.value.clone())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default())
                    .title(Span::styled(
                        "Value",
                        Style::default().add_modifier(Modifier::BOLD),
                    )),
            )
            .style(if focused {
                Style::default()
            } else {
                Style::default().fg(Color::DarkGray)
            })
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });
        f.render_widget(paragraph, area);
        Ok(())
    }
}

impl Component for TableValueComponent {
    fn event(&mut self, _key: Key) -> Result<EventState> {
        todo!("scroll");
    }
}
