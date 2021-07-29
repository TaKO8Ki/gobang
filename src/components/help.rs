use super::{Component, DrawableComponent, EventState};
use crate::components::command::CommandInfo;
use crate::event::Key;
use anyhow::Result;
use itertools::Itertools;
use std::convert::From;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
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
            let scroll_threshold = SIZE.1 / 3;
            let scroll = self.selection.saturating_sub(scroll_threshold);

            let area = Rect::new(
                (f.size().width.saturating_sub(SIZE.0)) / 2,
                (f.size().height.saturating_sub(SIZE.1)) / 2,
                SIZE.0.min(f.size().width),
                SIZE.1.min(f.size().height),
            );

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
                Paragraph::new(self.get_text(chunks[0])).scroll((scroll, 0)),
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
            match key {
                Key::Esc => {
                    self.hide();
                    return Ok(EventState::Consumed);
                }
                Key::Char('j') => {
                    self.move_selection(true);
                    return Ok(EventState::Consumed);
                }
                Key::Char('k') => {
                    self.move_selection(false);
                    return Ok(EventState::Consumed);
                }
                _ => (),
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

    fn move_selection(&mut self, inc: bool) {
        let mut new_selection = self.selection;

        new_selection = if inc {
            new_selection.saturating_add(1)
        } else {
            new_selection.saturating_sub(1)
        };
        new_selection = new_selection.max(0);

        self.selection = new_selection.min(self.cmds.len().saturating_sub(1) as u16);
    }

    fn get_text(&self, area: Rect) -> Vec<Spans> {
        let mut txt: Vec<Spans> = Vec::new();

        let mut processed = 0;

        for (key, group) in &self.cmds.iter().group_by(|e| e.text.group) {
            txt.push(Spans::from(Span::styled(
                key.to_string(),
                Style::default().add_modifier(Modifier::REVERSED),
            )));

            for command_info in group {
                let is_selected = self.selection == processed;
                processed += 1;

                txt.push(Spans::from(Span::styled(
                    format!("{}{:w$},", command_info.text.name, w = area.width as usize),
                    if is_selected {
                        Style::default().bg(Color::Blue)
                    } else {
                        Style::default()
                    },
                )));
            }
        }

        txt
    }
}