use super::{
    utils::scroll_vertical::VerticalScroll, Component, DrawableComponent, EventState,
    TableValueComponent,
};
use crate::event::Key;
use anyhow::Result;
use std::convert::From;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame,
};
use unicode_width::UnicodeWidthStr;

pub struct TableComponent {
    pub state: TableState,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub eod: bool,
    select_state: TableState,
    selected_left_column_index: usize,
    selected_right_cell: Option<(usize, usize)>,
    column_page_start: std::cell::Cell<usize>,
    scroll: VerticalScroll,
    select_entire_row: bool,
}

impl Default for TableComponent {
    fn default() -> Self {
        Self {
            state: TableState::default(),
            select_state: TableState::default(),
            headers: vec![],
            rows: vec![],
            selected_left_column_index: 0,
            selected_right_cell: None,
            column_page_start: std::cell::Cell::new(0),
            scroll: VerticalScroll::new(),
            select_entire_row: false,
            eod: false,
        }
    }
}

impl TableComponent {
    pub fn new(rows: Vec<Vec<String>>, headers: Vec<String>) -> Self {
        let mut state = TableState::default();
        if !rows.is_empty() {
            state.select(None);
            state.select(Some(0))
        }
        Self {
            rows,
            headers,
            state,
            ..Self::default()
        }
    }

    pub fn reset(&mut self) {
        self.select_state.select(None);
        self.selected_right_cell = None;
        self.select_entire_row = false;
    }

    pub fn end(&mut self) {
        self.eod = true;
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
        self.reset();
        self.state.select(i);
    }

    pub fn next_select(&mut self, lines: usize) {
        let i = match self.select_state.selected() {
            Some(i) => {
                if i + lines >= self.rows.len() {
                    Some(self.rows.len() - 1)
                } else {
                    Some(i + lines)
                }
            }
            None => None,
        };
        self.select_state.select(i);
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
        self.reset();
        self.state.select(i);
    }

    pub fn previout_select(&mut self, lines: usize) {
        let i = match self.select_state.selected() {
            Some(i) => {
                if i <= lines {
                    Some(0)
                } else {
                    Some(i - lines)
                }
            }
            None => None,
        };
        self.select_state.select(i);
    }

    pub fn scroll_top(&mut self) {
        if self.rows.is_empty() {
            return;
        }
        self.reset();
        self.state.select(Some(0));
    }

    pub fn scroll_bottom(&mut self) {
        if self.rows.is_empty() {
            return;
        }
        self.reset();
        self.state.select(Some(self.rows.len() - 1));
    }

    pub fn next_column(&mut self) {
        if self.rows.is_empty() {
            return;
        }
        if self.selected_left_column_index >= self.headers.len().saturating_sub(1) {
            return;
        }
        self.reset();
        self.selected_left_column_index += 1;
    }

    pub fn previous_column(&mut self) {
        if self.rows.is_empty() {
            return;
        }
        if self.selected_left_column_index == 0 {
            return;
        }
        self.reset();
        self.selected_left_column_index -= 1;
    }

    pub fn expand_selected_area_x(&mut self, positive: bool) {
        if self.selected_right_cell.is_none() {
            self.selected_right_cell = Some((
                self.selected_left_column_index,
                self.state.selected().unwrap_or(0),
            ));
        }
        if let Some((x, y)) = self.selected_right_cell {
            self.selected_right_cell = Some((
                if positive {
                    (x + 1).min(self.headers.len().saturating_sub(1))
                } else {
                    x.saturating_sub(1)
                },
                y,
            ));
        }
    }

    pub fn expand_selected_area_y(&mut self, positive: bool) {
        if self.select_state.selected().is_none() {
            self.select_state = self.state.clone();
        }

        if positive {
            self.next_select(1);
        } else {
            self.previout_select(1);
        }

        if self.selected_right_cell.is_none() {
            self.selected_right_cell = Some((
                self.selected_left_column_index,
                self.state.selected().unwrap_or(0),
            ));
        }
        if let Some((x, y)) = self.selected_right_cell {
            self.selected_right_cell = Some((
                x,
                if positive {
                    (y + 1).min(self.rows.len().saturating_sub(1))
                } else {
                    y.saturating_sub(1)
                },
            ));
        }
    }

    pub fn is_row_number_clumn(&self, row_index: usize, column_index: usize) -> bool {
        matches!(self.state.selected(), Some(selected_row_index) if row_index == selected_row_index && 0 == column_index)
    }

    pub fn selected_cells(&self) -> Option<String> {
        if let Some((x, y)) = self.selected_right_cell {
            let selected_row_index = self.state.selected()?;
            return Some(
                self.rows[y.min(selected_row_index)..y.max(selected_row_index) + 1]
                    .iter()
                    .map(|row| {
                        row[x.min(self.selected_left_column_index)
                            ..x.max(self.selected_left_column_index) + 1]
                            .join(",")
                            .to_string()
                    })
                    .collect::<Vec<String>>()
                    .join("\n"),
            );
        }
        self.rows
            .get(self.state.selected()?)?
            .get(self.selected_left_column_index)
            .map(|cell| cell.to_string())
    }

    pub fn selected_column_index(&self) -> usize {
        if let Some((x, _)) = self.selected_right_cell {
            return x;
        }
        self.selected_left_column_index
    }

    pub fn is_selected_cell(
        &self,
        row_index: usize,
        column_index: usize,
        selected_column_index: usize,
    ) -> bool {
        if let Some((x, y)) = self.selected_right_cell {
            let x_in_page = x
                .saturating_add(1)
                .saturating_sub(self.column_page_start.get());
            return matches!(
            self.state.selected(),
            Some(selected_row_index)
                if (x_in_page.min(selected_column_index).max(1)..x_in_page.max(selected_column_index) + 1)
                    .contains(&column_index)
                    && (y.min(selected_row_index)..y.max(selected_row_index) + 1)
                        .contains(&row_index)
            );
        }
        matches!(self.state.selected(), Some(selected_row_index) if row_index == selected_row_index &&  column_index == selected_column_index)
    }

    pub fn headers_with_number(&self, left: usize, right: usize) -> Vec<String> {
        let mut headers = self.headers.clone()[left..right].to_vec();
        headers.insert(0, "".to_string());
        headers
    }

    pub fn rows_with_number(&self, left: usize, right: usize) -> Vec<Vec<String>> {
        let rows = self
            .rows
            .iter()
            .map(|row| row.to_vec())
            .collect::<Vec<Vec<String>>>();
        let mut new_rows: Vec<Vec<String>> =
            rows.iter().map(|row| row[left..right].to_vec()).collect();
        for (index, row) in new_rows.iter_mut().enumerate() {
            row.insert(0, (index + 1).to_string())
        }
        new_rows
    }

    pub fn calculate_widths(
        &self,
        area_width: u16,
    ) -> (usize, Vec<String>, Vec<Vec<String>>, Vec<Constraint>) {
        if self.rows.is_empty() {
            return (0, Vec::new(), Vec::new(), Vec::new());
        }
        if self.selected_column_index() < self.column_page_start.get() {
            self.column_page_start.set(self.selected_column_index());
        }

        let right_column_index = self.selected_column_index().clone();
        let mut column_index = self.selected_column_index();
        let number_clomn_width = (self.rows.len() + 1).to_string().width() as u16;
        let mut widths = vec![];
        loop {
            let length = self
                .rows
                .iter()
                .map(|row| {
                    row.get(column_index)
                        .map_or(String::new(), |cell| cell.to_string())
                        .width()
                })
                .collect::<Vec<usize>>()
                .iter()
                .max()
                .map_or(3, |v| {
                    *v.max(
                        &self
                            .headers
                            .get(column_index)
                            .map_or(3, |header| header.to_string().width()),
                    )
                    .clamp(&3, &20)
                });
            if widths.iter().map(|(_, width)| width).sum::<usize>() + length + widths.len() * 1
                > area_width.saturating_sub(number_clomn_width) as usize
            {
                column_index += 1;
                break;
            }
            widths.push((self.headers[column_index].clone(), length));
            if column_index == self.column_page_start.get() {
                break;
            }
            column_index -= 1;
        }
        let left_column_index = column_index;
        widths.reverse();
        let selected_column_index = widths.len().saturating_sub(1);
        let mut column_index = right_column_index + 1;
        while widths.iter().map(|(_, width)| width).sum::<usize>() + widths.len() * 1
            <= area_width.saturating_sub(number_clomn_width) as usize
        {
            let length = self
                .rows
                .iter()
                .map(|row| {
                    row.get(column_index)
                        .map_or(String::new(), |cell| cell.to_string())
                        .width()
                })
                .collect::<Vec<usize>>()
                .iter()
                .max()
                .map_or(3, |v| {
                    *v.max(
                        self.headers
                            .iter()
                            .map(|header| header.to_string().width())
                            .collect::<Vec<usize>>()
                            .get(column_index)
                            .unwrap_or(&3),
                    )
                    .clamp(&3, &20)
                });
            match self.headers.get(column_index) {
                Some(header) => {
                    widths.push((header.to_string(), length));
                }
                None => break,
            }
            column_index += 1
        }
        if self.selected_column_index() != self.headers.len().saturating_sub(1) {
            widths.pop();
        }
        let right_column_index = column_index;
        let mut constraints = widths
            .iter()
            .map(|(_, width)| Constraint::Length(*width as u16))
            .collect::<Vec<Constraint>>();
        if self.selected_column_index() != self.headers.len().saturating_sub(1) {
            constraints.push(Constraint::Min(10));
        }
        constraints.insert(0, Constraint::Length(number_clomn_width));
        self.column_page_start.set(left_column_index);
        (
            self.selected_right_cell
                .map_or(selected_column_index + 1, |(x, _)| {
                    if x > self.selected_left_column_index {
                        (selected_column_index + 1)
                            .saturating_sub(x.saturating_sub(self.selected_left_column_index))
                    } else {
                        (selected_column_index + 1)
                            .saturating_add(self.selected_left_column_index.saturating_sub(x))
                    }
                }),
            self.headers_with_number(left_column_index, right_column_index),
            self.rows_with_number(left_column_index, right_column_index),
            constraints,
        )
    }
}

impl DrawableComponent for TableComponent {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect, focused: bool) -> Result<()> {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(3), Constraint::Length(5)])
            .split(area);

        self.state.selected().map_or_else(
            || {
                self.scroll.reset();
            },
            |selection| {
                self.scroll.update(
                    selection,
                    self.rows.len(),
                    layout[1].height.saturating_sub(2) as usize,
                );
            },
        );

        TableValueComponent::new(self.selected_cells().unwrap_or_default())
            .draw(f, layout[0], focused)?;

        let block = Block::default().borders(Borders::ALL).title("Records");
        let (selected_column_index, headers, rows, constraints) =
            self.calculate_widths(block.inner(layout[1]).width);
        let header_cells = headers.iter().enumerate().map(|(column_index, h)| {
            Cell::from(h.to_string()).style(if selected_column_index == column_index {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            })
        });
        let header = Row::new(header_cells).height(1).bottom_margin(1);
        let rows = rows.iter().enumerate().map(|(row_index, item)| {
            let height = item
                .iter()
                .map(|content| content.chars().filter(|c| *c == '\n').count())
                .max()
                .unwrap_or(0)
                + 1;
            let cells = item.iter().enumerate().map(|(column_index, c)| {
                Cell::from(c.to_string()).style(
                    if self.is_selected_cell(row_index, column_index, selected_column_index) {
                        Style::default().bg(Color::Blue)
                    } else if self.is_row_number_clumn(row_index, column_index) {
                        Style::default().add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    },
                )
            });
            Row::new(cells).height(height as u16).bottom_margin(1)
        });

        let table = Table::new(rows)
            .header(header)
            .block(block)
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
            .widths(&constraints);
        f.render_stateful_widget(
            table,
            layout[1],
            if self.select_state.selected().is_some() {
                &mut self.select_state
            } else {
                &mut self.state
            },
        );

        self.scroll.draw(f, layout[1]);
        Ok(())
    }
}

impl Component for TableComponent {
    fn event(&mut self, key: Key) -> Result<EventState> {
        match key {
            Key::Char('h') => {
                self.previous_column();
                return Ok(EventState::Consumed);
            }
            Key::Char('j') => {
                self.next(1);
                return Ok(EventState::NotConsumed);
            }
            Key::Ctrl('d') => {
                self.next(10);
                return Ok(EventState::NotConsumed);
            }
            Key::Char('k') => {
                self.previous(1);
                return Ok(EventState::Consumed);
            }
            Key::Ctrl('u') => {
                self.previous(10);
                return Ok(EventState::Consumed);
            }
            Key::Char('g') => {
                self.scroll_top();
                return Ok(EventState::Consumed);
            }
            Key::Char('r') => {
                self.select_entire_row = true;
                return Ok(EventState::Consumed);
            }
            Key::Char('G') => {
                self.scroll_bottom();
                return Ok(EventState::Consumed);
            }
            Key::Char('l') => {
                self.next_column();
                return Ok(EventState::Consumed);
            }
            Key::Char('H') => {
                self.expand_selected_area_x(false);
                return Ok(EventState::Consumed);
            }
            Key::Char('K') => {
                self.expand_selected_area_y(false);
                return Ok(EventState::Consumed);
            }
            Key::Char('J') => {
                self.expand_selected_area_y(true);
                return Ok(EventState::Consumed);
            }
            Key::Char('L') => {
                self.expand_selected_area_x(true);
                return Ok(EventState::Consumed);
            }
            _ => (),
        }
        Ok(EventState::NotConsumed)
    }
}

#[cfg(test)]
mod test {
    use super::TableComponent;
    use tui::layout::Constraint;

    #[test]
    fn test_headers() {
        let mut component = TableComponent::default();
        component.headers = vec!["a", "b", "c"].iter().map(|h| h.to_string()).collect();
        assert_eq!(component.headers_with_number(1, 2), vec!["", "b"])
    }

    #[test]
    fn test_rows() {
        let mut component = TableComponent::default();
        component.rows = vec![
            vec!["a", "b", "c"].iter().map(|h| h.to_string()).collect(),
            vec!["d", "e", "f"].iter().map(|h| h.to_string()).collect(),
        ];
        assert_eq!(
            component.rows_with_number(1, 2),
            vec![vec!["1", "b"], vec!["2", "e"]],
        )
    }

    #[test]
    fn test_expand_selected_area_x_left() {
        // before
        //    1  2  3
        // 1  a  b  c
        // 2  d |e| f

        // after
        //    1  2  3
        // 1  a  b  c
        // 2 |d  e| f

        let mut component = TableComponent::default();
        component.headers = vec!["1", "2", "3"].iter().map(|h| h.to_string()).collect();
        component.rows = vec![
            vec!["a", "b", "c"].iter().map(|h| h.to_string()).collect(),
            vec!["d", "e", "f"].iter().map(|h| h.to_string()).collect(),
        ];
        component.state.select(Some(1));
        component.selected_left_column_index = 1;
        component.expand_selected_area_x(false);
        assert_eq!(component.selected_right_cell, Some((0, 1)));
        assert_eq!(component.selected_cells(), Some("d,e".to_string()));
    }

    #[test]
    fn test_expand_selected_area_x_right() {
        // before
        //    1  2  3
        // 1  a  b  c
        // 2  d |e| f

        // after
        //    1  2  3
        // 1  a  b  c
        // 2  d |e  f|

        let mut component = TableComponent::default();
        component.headers = vec!["1", "2", "3"].iter().map(|h| h.to_string()).collect();
        component.rows = vec![
            vec!["a", "b", "c"].iter().map(|h| h.to_string()).collect(),
            vec!["d", "e", "f"].iter().map(|h| h.to_string()).collect(),
        ];
        component.state.select(Some(1));
        component.selected_left_column_index = 1;
        component.expand_selected_area_x(true);
        assert_eq!(component.selected_right_cell, Some((2, 1)));
        assert_eq!(component.selected_cells(), Some("e,f".to_string()));
    }

    #[test]
    fn test_expand_selected_area_y_up() {
        // before
        //    1  2  3
        // 1  a  b  c
        // 2  d |e| f

        // after
        //    1  2  3
        // 1  a |b| c
        // 2  d |e| f

        let mut component = TableComponent::default();
        component.rows = vec![
            vec!["a", "b", "c"].iter().map(|h| h.to_string()).collect(),
            vec!["d", "e", "f"].iter().map(|h| h.to_string()).collect(),
        ];
        component.state.select(Some(1));
        component.selected_left_column_index = 1;
        component.expand_selected_area_y(false);
        assert_eq!(component.selected_right_cell, Some((1, 0)));
        assert_eq!(component.selected_cells(), Some("b\ne".to_string()));
    }

    #[test]
    fn test_expand_selected_area_y_down() {
        // before
        //    1  2  3
        // 1  a |b| c
        // 2  d  e  f

        // after
        //    1  2  3
        // 1  a |b| c
        // 2  d |e| f

        let mut component = TableComponent::default();
        component.rows = vec![
            vec!["a", "b", "c"].iter().map(|h| h.to_string()).collect(),
            vec!["d", "e", "f"].iter().map(|h| h.to_string()).collect(),
        ];
        component.state.select(Some(0));
        component.selected_left_column_index = 1;
        component.expand_selected_area_y(true);
        assert_eq!(component.selected_right_cell, Some((1, 1)));
        assert_eq!(component.selected_cells(), Some("b\ne".to_string()));
    }

    #[test]
    fn test_selected_cell_when_one_cell_selected() {
        //    1  2 3
        // 1 |a| b c
        // 2  d  e f

        let mut component = TableComponent::default();
        component.headers = vec!["1", "2", "3"].iter().map(|h| h.to_string()).collect();
        component.rows = vec![
            vec!["a", "b", "c"].iter().map(|h| h.to_string()).collect(),
            vec!["d", "e", "f"].iter().map(|h| h.to_string()).collect(),
        ];
        component.state.select(Some(0));
        assert_eq!(component.selected_cells(), Some("a".to_string()));
    }

    #[test]
    fn test_selected_cell_when_multiple_cells_selected() {
        //    1  2  3
        // 1 |a  b| c
        // 2 |d  e| f

        let mut component = TableComponent::default();
        component.headers = vec!["1", "2", "3"].iter().map(|h| h.to_string()).collect();
        component.rows = vec![
            vec!["a", "b", "c"].iter().map(|h| h.to_string()).collect(),
            vec!["d", "e", "f"].iter().map(|h| h.to_string()).collect(),
        ];
        component.state.select(Some(0));
        component.selected_right_cell = Some((1, 1));
        assert_eq!(component.selected_cells(), Some("a,b\nd,e".to_string()));
    }

    #[test]
    fn test_is_selected_cell_when_one_cell_selected() {
        //    1  2 3
        // 1 |a| b c
        // 2  d  e f

        let mut component = TableComponent::default();
        component.headers = vec!["1", "2", "3"].iter().map(|h| h.to_string()).collect();
        component.rows = vec![
            vec!["a", "b", "c"].iter().map(|h| h.to_string()).collect(),
            vec!["d", "e", "f"].iter().map(|h| h.to_string()).collect(),
        ];
        component.state.select(Some(0));
        // a
        assert!(component.is_selected_cell(0, 1, 1));
        // d
        assert!(!component.is_selected_cell(1, 1, 1));
        // e
        assert!(!component.is_selected_cell(1, 2, 1));
    }

    #[test]
    fn test_is_selected_cell_when_multiple_cells_selected() {
        //    1  2  3
        // 1 |a  b| c
        // 2 |d  e| f

        let mut component = TableComponent::default();
        component.headers = vec!["1", "2", "3"].iter().map(|h| h.to_string()).collect();
        component.rows = vec![
            vec!["a", "b", "c"].iter().map(|h| h.to_string()).collect(),
            vec!["d", "e", "f"].iter().map(|h| h.to_string()).collect(),
        ];
        component.state.select(Some(0));
        component.selected_right_cell = Some((1, 1));
        // a
        assert!(component.is_selected_cell(0, 1, 1));
        // b
        assert!(component.is_selected_cell(0, 2, 1));
        // d
        assert!(component.is_selected_cell(1, 1, 1));
        // e
        assert!(component.is_selected_cell(1, 2, 1));
        // f
        assert!(!component.is_selected_cell(1, 3, 1));
    }

    #[test]
    fn test_calculate_widths() {
        let mut component = TableComponent::default();
        component.headers = vec!["1", "2", "3"].iter().map(|h| h.to_string()).collect();
        component.rows = vec![
            vec!["aaaaa", "bbbbb", "ccccc"]
                .iter()
                .map(|h| h.to_string())
                .collect(),
            vec!["d", "e", "f"].iter().map(|h| h.to_string()).collect(),
        ];
        let (selected_column_index, headers, rows, constraints) = component.calculate_widths(10);
        assert_eq!(selected_column_index, 1);
        assert_eq!(headers, vec!["", "1", "2"]);
        assert_eq!(rows, vec![vec!["1", "aaaaa", "bbbbb"], vec!["2", "d", "e"]]);
        assert_eq!(
            constraints,
            vec![
                Constraint::Length(1),
                Constraint::Length(5),
                Constraint::Min(10),
            ]
        );

        let (selected_column_index, headers, rows, constraints) = component.calculate_widths(20);
        assert_eq!(selected_column_index, 1);
        assert_eq!(headers, vec!["", "1", "2", "3"]);
        assert_eq!(
            rows,
            vec![
                vec!["1", "aaaaa", "bbbbb", "ccccc"],
                vec!["2", "d", "e", "f"]
            ]
        );
        assert_eq!(
            constraints,
            vec![
                Constraint::Length(1),
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Min(10),
            ]
        );
    }
}
