use super::{Component, DrawableComponent};
use crate::event::Key;
use anyhow::Result;
use tui::{
    backend::Backend,
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub struct ErrorComponent {
    pub error: Option<String>,
}

impl Default for ErrorComponent {
    fn default() -> Self {
        Self { error: None }
    }
}

impl ErrorComponent {
    pub fn set(&mut self, error: String) {
        self.error = Some(error);
    }
}

impl DrawableComponent for ErrorComponent {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, _area: Rect, _focused: bool) -> Result<()> {
        if let Some(error) = self.error.as_ref() {
            let width = 65;
            let height = 10;
            let error = Paragraph::new(error.to_string())
                .block(Block::default().title("Error").borders(Borders::ALL))
                .style(Style::default().fg(Color::Red))
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: true });
            let area = Rect::new(
                (f.size().width.saturating_sub(width)) / 2,
                (f.size().height.saturating_sub(height)) / 2,
                width.min(f.size().width),
                height.min(f.size().height),
            );
            f.render_widget(Clear, area);
            f.render_widget(error, area);
        }
        Ok(())
    }
}

impl Component for ErrorComponent {
    fn event(&mut self, key: Key) -> Result<()> {
        Ok(())
    }
}
