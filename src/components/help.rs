use super::{Component, DrawableComponent, EventState};
use crate::components::command::CommandInfo;
use crate::event::Key;
use anyhow::Result;
use itertools::Itertools;
use std::convert::From;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

pub struct HelpComponent {
    cmds: Vec<CommandInfo>,
    visible: bool,
    selection: u16,
}

impl DrawableComponent for HelpComponent {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, _area: Rect, _focused: bool) -> Result<()> {
        if self.visible {
            const SIZE: (u16, u16) = (65, 24);

            let area = Rect::new(
                (f.size().width.saturating_sub(SIZE.0)) / 2,
                (f.size().height.saturating_sub(SIZE.1)) / 2,
                SIZE.0.min(f.size().width),
                SIZE.1.min(f.size().height),
            );

            let scroll = 0;

            f.render_widget(Clear, area);
            f.render_widget(
                Block::default()
                    .title("Help")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Thick),
                area,
            );

            let chunks = Layout::default()
                .vertical_margin(1)
                .horizontal_margin(1)
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(1)].as_ref())
                .split(area);

            f.render_widget(
                Paragraph::new(self.get_text()).scroll((scroll, 0)),
                chunks[0],
            );

            f.render_widget(
                Paragraph::new(Spans::from(vec![Span::styled(
                    format!("gobang {}", "0.1.0"),
                    Style::default(),
                )]))
                .alignment(Alignment::Right),
                chunks[1],
            );
        }

        Ok(())
    }
}

impl Component for HelpComponent {
    fn commands(&self, out: &mut Vec<CommandInfo>) {}

    fn event(&mut self, key: Key) -> Result<EventState> {
        if self.visible {
            if let Key::Esc = key {
                self.hide();
                return Ok(EventState::Consumed);
            }
            return Ok(EventState::NotConsumed);
        } else if let Key::Char('?') = key {
            self.show()?;
            return Ok(EventState::Consumed);
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

impl HelpComponent {
    pub const fn new() -> Self {
        Self {
            cmds: vec![],
            visible: false,
            selection: 0,
        }
    }

    pub fn set_cmds(&mut self, cmds: Vec<CommandInfo>) {
        self.cmds = cmds
            .into_iter()
            .filter(|e| !e.text.hide_help)
            .collect::<Vec<_>>();
    }

    fn get_text(&self) -> Vec<Spans> {
        let mut txt: Vec<Spans> = Vec::new();

        for (key, group) in &self.cmds.iter().group_by(|e| e.text.group) {
            txt.push(Spans::from(Span::styled(
                key.to_string(),
                Style::default().add_modifier(Modifier::REVERSED),
            )));

            for command_info in group {
                txt.push(Spans::from(Span::styled(
                    format!("{}", command_info.text.name),
                    Style::default(),
                )));
            }
        }

        txt
    }
}
