use super::{utils::scroll_vertical::VerticalScroll, Component, DrawableComponent};
use crate::event::Key;
use anyhow::Result;
use std::convert::From;
use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame,
};

pub struct TableComponent {
    pub state: TableState,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub column_index: usize,
    pub column_page: usize,
    pub scroll: VerticalScroll,
    pub select_entire_row: bool,
}

impl Default for TableComponent {
    fn default() -> Self {
        Self {
            state: TableState::default(),
            headers: vec![],
            rows: vec![],
            column_page: 0,
            column_index: 0,
            scroll: VerticalScroll::new(),
            select_entire_row: false,
        }
    }
}

impl TableComponent {
    pub fn reset(&mut self, headers: Vec<String>, rows: Vec<Vec<String>>) {
        self.headers = headers;
        self.rows = rows;
        self.column_page = 0;
        self.column_index = 1;
        self.state.select(None);
        if !self.rows.is_empty() {
            self.state.select(Some(0));
        }
    }

    pub fn next(&mut self, lines: usize) {
        let i = match self.state.selected() {
            Some(i) => {
                if i + lines >= self.rows.len() {
                    Some(self.rows.len() - 1)
                } else {
                    Some(i + lines)
                }
            }
            None => None,
        };
        self.select_entire_row = false;
        self.state.select(i);
    }

    pub fn previous(&mut self, lines: usize) {
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
        self.select_entire_row = false;
        self.state.select(i);
    }

    pub fn scroll_top(&mut self) {
        if self.rows.is_empty() {
            return;
        }
        self.state.select(None);
        self.state.select(Some(0));
    }

    pub fn scroll_bottom(&mut self) {
        if self.rows.is_empty() {
            return;
        }
        self.state.select(Some(self.rows.len() - 1));
    }

    pub fn next_column(&mut self) {
        if self.rows.is_empty() {
            return;
        }
        if self.column_index == self.headers.len() - 1 {
            return;
        }
        if self.column_index == 9 {
            self.next_column_page();
            return;
        }
        self.select_entire_row = false;
        self.column_index += 1;
    }

    pub fn previous_column(&mut self) {
        if self.rows.is_empty() {
            return;
        }
        if self.column_index == 1 {
            self.previous_column_page();
            return;
        }
        self.select_entire_row = false;
        self.column_index -= 1;
    }

    pub fn next_column_page(&mut self) {
        if self.headers.len() > 9 && self.column_page < self.headers.len() - 9 {
            self.column_page += 1
        }
    }

    pub fn previous_column_page(&mut self) {
        if self.column_page > 0 {
            self.column_page -= 1
        }
    }

    pub fn headers(&self) -> Vec<String> {
        let mut headers = self.headers[self.column_page..].to_vec();
        headers.insert(0, "".to_string());
        headers
    }

    pub fn rows(&self) -> Vec<Vec<String>> {
        let rows = self
            .rows
            .iter()
            .map(|row| row[self.column_page..].to_vec())
            .collect::<Vec<Vec<String>>>();
        let mut new_rows = match self.state.selected() {
            Some(index) => {
                if index + 100 <= self.rows.len() {
                    rows[..index + 100].to_vec()
                } else {
                    rows
                }
            }
            None => rows,
        };
        for (index, row) in new_rows.iter_mut().enumerate() {
            row.insert(0, (index + 1).to_string())
        }
        new_rows
    }
}

impl DrawableComponent for TableComponent {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect, focused: bool) -> Result<()> {
        self.state.selected().map_or_else(
            || {
                self.scroll.reset();
            },
            |selection| {
                self.scroll.update(
                    selection,
                    self.rows.len(),
                    area.height.saturating_sub(2) as usize,
                );
            },
        );

        let headers = self.headers();
        let header_cells = headers
            .iter()
            .map(|h| Cell::from(h.to_string()).style(Style::default()));
        let header = Row::new(header_cells).height(1).bottom_margin(1);
        let rows = self.rows();
        let rows = rows.iter().enumerate().map(|(row_index, item)| {
            let height = item
                .iter()
                .map(|content| content.chars().filter(|c| *c == '\n').count())
                .max()
                .unwrap_or(0)
                + 1;
            let cells = item.iter().enumerate().map(|(column_page, c)| {
                Cell::from(c.to_string()).style(if column_page == self.column_index {
                    match self.state.selected() {
                        Some(selected_row) => {
                            if row_index == selected_row {
                                Style::default().bg(Color::Blue)
                            } else {
                                Style::default()
                            }
                        }
                        None => Style::default(),
                    }
                } else {
                    Style::default()
                })
            });
            Row::new(cells).height(height as u16).bottom_margin(1)
        });
        let widths = (0..10)
            .map(|_| Constraint::Percentage(10))
            .collect::<Vec<Constraint>>();
        let t = Table::new(rows)
            .header(header)
            .block(Block::default().borders(Borders::ALL).title("Records"))
            .highlight_style(if self.select_entire_row {
                Style::default().bg(Color::Blue)
            } else {
                Style::default()
            })
            .style(if focused {
                Style::default()
            } else {
                Style::default().fg(Color::DarkGray)
            })
            .widths(&widths);
        f.render_stateful_widget(t, area, &mut self.state);

        self.scroll.draw(f, area);
        Ok(())
    }
}

impl Component for TableComponent {
    fn event(&mut self, key: Key) -> Result<()> {
        match key {
            Key::Char('h') => self.previous_column(),
            Key::Char('j') => self.next(1),
            Key::Ctrl('d') => self.next(10),
            Key::Char('k') => self.previous(1),
            Key::Ctrl('u') => self.previous(10),
            Key::Char('g') => self.scroll_top(),
            Key::Char('r') => self.select_entire_row = true,
            Key::Shift('G') | Key::Shift('g') => self.scroll_bottom(),
            Key::Char('l') => self.next_column(),
            _ => (),
        }
        Ok(())
    }
}
