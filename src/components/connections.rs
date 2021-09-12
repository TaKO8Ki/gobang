use super::{Component, DrawableComponent, EventState};
use crate::components::command::CommandInfo;
use crate::config::{Connection, KeyConfig};
use crate::event::Key;
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
    key_config: KeyConfig,
}

impl ConnectionsComponent {
    pub fn new(key_config: KeyConfig, connections: Vec<Connection>) -> Self {
        let mut state = ListState::default();
        if !connections.is_empty() {
            state.select(Some(0));
        }
        Self {
            connections,
            key_config,
            state,
        }
    }

    fn next_connection(&mut self, lines: usize) {
        let i = match self.state.selected() {
            Some(i) => {
                if i + lines >= self.connections.len() {
                    Some(self.connections.len() - 1)
                } else {
                    Some(i + lines)
                }
            }
            None => None,
        };
        self.state.select(i);
    }

    fn previous_connection(&mut self, lines: usize) {
        let i = match self.state.selected() {
            Some(i) => {
                if i <= lines {
                    Some(0)
                } else {
                    Some(i - lines)
                }
            }
            None => None,
        };
        self.state.select(i);
    }

    fn scroll_to_top(&mut self) {
        if self.connections.is_empty() {
            return;
        }
        self.state.select(Some(0));
    }

    fn scroll_to_bottom(&mut self) {
        if self.connections.is_empty() {
            return;
        }
        self.state.select(Some(self.connections.len() - 1));
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
        let mut connections: Vec<ListItem> = Vec::new();
        for c in conns {
            connections.push(
                ListItem::new(vec![Spans::from(Span::raw(c.database_url()?))])
                    .style(Style::default()),
            )
        }
        let connections = List::new(connections)
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
        f.render_stateful_widget(connections, area, &mut self.state);
        Ok(())
    }
}

impl Component for ConnectionsComponent {
    fn commands(&self, _out: &mut Vec<CommandInfo>) {}

    fn event(&mut self, key: Key) -> Result<EventState> {
        if key == self.key_config.scroll_down {
            self.next_connection(1);
            return Ok(EventState::Consumed);
        } else if key == self.key_config.scroll_up {
            self.previous_connection(1);
            return Ok(EventState::Consumed);
        } else if key == self.key_config.scroll_down_multiple_lines {
            self.next_connection(10);
            return Ok(EventState::NotConsumed);
        } else if key == self.key_config.scroll_up_multiple_lines {
            self.previous_connection(10);
            return Ok(EventState::Consumed);
        } else if key == self.key_config.scroll_to_top {
            self.scroll_to_top();
            return Ok(EventState::Consumed);
        } else if key == self.key_config.scroll_to_bottom {
            self.scroll_to_bottom();
            return Ok(EventState::Consumed);
        }
        Ok(EventState::NotConsumed)
    }
}
