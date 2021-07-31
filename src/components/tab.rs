use super::{Component, DrawableComponent, EventState};
use crate::components::command::{self, CommandInfo};
use crate::config::KeyConfig;
use crate::event::Key;
use anyhow::Result;
use strum_macros::EnumIter;
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Spans,
    widgets::{Block, Borders, Tabs},
    Frame,
};

#[derive(Debug, Clone, Copy, EnumIter)]
pub enum Tab {
    Records,
    Structure,
}

impl std::fmt::Display for Tab {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct TabComponent {
    pub selected_tab: Tab,
    key_config: KeyConfig,
}

impl TabComponent {
    pub fn new(key_config: KeyConfig) -> Self {
        Self {
            selected_tab: Tab::Records,
            key_config,
        }
    }

    fn names(&self) -> Vec<String> {
        vec![
            command::tab_records(&self.key_config).name,
            command::tab_structure(&self.key_config).name,
        ]
    }
}

impl DrawableComponent for TabComponent {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect, _focused: bool) -> Result<()> {
        let titles = self.names().iter().cloned().map(Spans::from).collect();
        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL))
            .select(self.selected_tab as usize)
            .style(Style::default().fg(Color::DarkGray))
            .highlight_style(
                Style::default()
                    .fg(Color::Reset)
                    .add_modifier(Modifier::UNDERLINED),
            );
        f.render_widget(tabs, area);
        Ok(())
    }
}

impl Component for TabComponent {
    fn commands(&self, _out: &mut Vec<CommandInfo>) {}

    fn event(&mut self, key: Key) -> Result<EventState> {
        match key {
            Key::Char('1') => {
                self.selected_tab = Tab::Records;
                Ok(EventState::Consumed)
            }
            Key::Char('2') => {
                self.selected_tab = Tab::Structure;
                Ok(EventState::Consumed)
            }
            _ => Ok(EventState::NotConsumed),
        }
    }
}
