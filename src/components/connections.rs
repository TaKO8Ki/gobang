use super::{Component, DrawableComponent};
use crate::event::Key;
use crate::user_config::Connection;
use anyhow::Result;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
    Frame,
};

pub struct ConnectionsComponent {
    pub connections: Vec<Connection>,
    pub state: ListState,
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

    pub fn next_connection(&mut self) {
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

    pub fn previous_connection(&mut self) {
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
}

impl DrawableComponent for ConnectionsComponent {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, _area: Rect, focused: bool) -> Result<()> {
        let percent_x = 60;
        let percent_y = 50;
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
            .style(if focused {
                Style::default()
            } else {
                Style::default().fg(Color::DarkGray)
            });
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage((100 - percent_y) / 2),
                    Constraint::Percentage(percent_y),
                    Constraint::Percentage((100 - percent_y) / 2),
                ]
                .as_ref(),
            )
            .split(f.size());

        let area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage((100 - percent_x) / 2),
                    Constraint::Percentage(percent_x),
                    Constraint::Percentage((100 - percent_x) / 2),
                ]
                .as_ref(),
            )
            .split(popup_layout[1])[1];
        f.render_widget(Clear, area);
        f.render_stateful_widget(tasks, area, &mut self.state);
        return Ok(());
    }
}

impl Component for ConnectionsComponent {
    fn event(&mut self, key: Key) -> Result<()> {
        match key {
            Key::Char('j') => self.next_connection(),
            Key::Char('k') => self.previous_connection(),
            _ => (),
        }
        Ok(())
    }
}
