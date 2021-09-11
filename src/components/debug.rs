use super::{Component, DrawableComponent, EventState};
use crate::components::command::CommandInfo;
use crate::config::KeyConfig;
use crate::event::Key;
use anyhow::Result;
use tui::{
    backend::Backend,
    layout::{Alignment, Rect},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub struct DebugComponent {
    msg: String,
    visible: bool,
    key_config: KeyConfig,
}

impl DebugComponent {
    #[allow(dead_code)]
    pub fn new(key_config: KeyConfig, msg: String) -> Self {
        Self {
            msg,
            visible: false,
            key_config,
        }
    }
}

impl DrawableComponent for DebugComponent {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, _area: Rect, _focused: bool) -> Result<()> {
        if true {
            let width = 65;
            let height = 10;
            let error = Paragraph::new(self.msg.to_string())
                .block(Block::default().title("Debug").borders(Borders::ALL))
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

impl Component for DebugComponent {
    fn commands(&self, _out: &mut Vec<CommandInfo>) {}

    fn event(&mut self, key: Key) -> Result<EventState> {
        if self.visible {
            if key == self.key_config.exit_popup {
                self.msg = String::new();
                self.hide();
                return Ok(EventState::Consumed);
            }
            return Ok(EventState::NotConsumed);
        }
        Ok(EventState::NotConsumed)
    }

    fn hide(&mut self) {
        self.visible = false;
    }

    fn show(&mut self) -> Result<()> {
        self.visible = true;

        Ok(())
    }
}
