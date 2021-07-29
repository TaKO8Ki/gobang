use super::{Component, DrawableComponent, EventState};
use crate::components::command::CommandInfo;
use crate::event::Key;
use crate::user_config::Connection;
use anyhow::Result;
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
    Frame,
};

pub struct ConnectionsComponent {
    connections: Vec<Connection>,
    state: ListState,
}

impl Default for ConnectionsComponent {
    fn default() -> Self {
        Self {
            connections: Vec::new(),
            state: ListState::default(),
        }
    }
}

impl ConnectionsComponent {
    pub fn new(connections: Vec<Connection>) -> Self {
        Self {
            connections,
            ..Self::default()
        }
    }

    fn next_connection(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.connections.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous_connection(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.connections.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn selected_connection(&self) -> Option<&Connection> {
        match self.state.selected() {
            Some(i) => self.connections.get(i),
            None => None,
        }
    }
}

impl DrawableComponent for ConnectionsComponent {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, _area: Rect, _focused: bool) -> Result<()> {
        let width = 80;
        let height = 20;
        let conns = &self.connections;
        let connections: Vec<ListItem> = conns
            .iter()
            .map(|i| {
                ListItem::new(vec![Spans::from(Span::raw(i.database_url()))])
                    .style(Style::default())
            })
            .collect();
        let tasks = List::new(connections)
            .block(Block::default().borders(Borders::ALL).title("Connections"))
            .highlight_style(Style::default().bg(Color::Blue))
            .style(Style::default());

        let area = Rect::new(
            (f.size().width.saturating_sub(width)) / 2,
            (f.size().height.saturating_sub(height)) / 2,
            width.min(f.size().width),
            height.min(f.size().height),
        );
        f.render_widget(Clear, area);
        f.render_stateful_widget(tasks, area, &mut self.state);
        Ok(())
    }
}

impl Component for ConnectionsComponent {
    fn commands(&self, out: &mut Vec<CommandInfo>) {}

    fn event(&mut self, key: Key) -> Result<EventState> {
        match key {
            Key::Char('j') => {
                self.next_connection();
                return Ok(EventState::Consumed);
            }
            Key::Char('k') => {
                self.previous_connection();
                return Ok(EventState::Consumed);
            }
            _ => (),
        }
        Ok(EventState::NotConsumed)
    }
}
