use super::{Component, DrawableComponent, EventState};
use crate::components::command::CommandInfo;
use crate::event::Key;
use anyhow::Result;
use strum::IntoEnumIterator;
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

impl Tab {
    pub fn names() -> Vec<String> {
        Self::iter()
            .map(|tab| format!("{} [{}]", tab, tab as u8 + 1))
            .collect()
    }
}

pub struct TabComponent {
    pub selected_tab: Tab,
}

impl Default for TabComponent {
    fn default() -> Self {
        Self {
            selected_tab: Tab::Records,
        }
    }
}

impl DrawableComponent for TabComponent {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect, _focused: bool) -> Result<()> {
        let titles = Tab::names().iter().cloned().map(Spans::from).collect();
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
    fn commands(&self, out: &mut Vec<CommandInfo>) {}

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
