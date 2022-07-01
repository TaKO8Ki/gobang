use super::{
    utils::scroll_vertical::VerticalScroll, Component, DrawableComponent, EventState,
    StatefulDrawableComponent, TableStatusComponent, TableValueComponent,
};
use crate::components::command::{self, CommandInfo};
use crate::config::KeyConfig;
use crate::event::Key;
use anyhow::Result;
use database_tree::{Database, Table as DTable};
use std::convert::From;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame,
};
use unicode_width::UnicodeWidthStr;

#[derive(Debug, PartialEq)]
struct Order {
    pub column_number: usize,
    pub is_asc: bool,
}

impl Order {
    pub fn new(column_number: usize, is_asc: bool) -> Self {
        Self {
            column_number,
            is_asc,
        }
    }

    fn query(&self) -> String {
        let order = if self.is_asc { "ASC" } else { "DESC" };

        return format!(
            "{column} {order}",
            column = self.column_number,
            order = order
        );
    }
}

#[derive(PartialEq)]
struct OrderManager {
    orders: Vec<Order>,
}

impl OrderManager {
    fn new() -> Self {
        Self { orders: vec![] }
    }

    fn generate_order_query(&mut self) -> Option<String> {
        let order_query = self
            .orders
            .iter()
            .map(|order| order.query())
            .collect::<Vec<String>>();

        if !order_query.is_empty() {
            return Some("ORDER BY ".to_string() + &order_query.join(", "));
        }

        None
    }

    fn generate_header_icons(&mut self, header_length: usize) -> Vec<String> {
        let mut header_icons = vec![String::new(); header_length];

        for (index, order) in self.orders.iter().enumerate() {
            let arrow = if order.is_asc { "↑" } else { "↓" };
            header_icons[order.column_number - 1] =
                format!("{arrow}{number}", arrow = arrow, number = index + 1);
        }

        header_icons
    }

    fn add_order(&mut self, selected_column: usize) {
        let selected_column_number = selected_column + 1;
        if let Some(position) = self
            .orders
            .iter()
            .position(|order| order.column_number == selected_column_number)
        {
            if self.orders[position].is_asc {
                self.orders[position].is_asc = false;
            } else {
                self.orders.remove(position);
            }
        } else {
            let order = Order::new(selected_column_number, true);
            self.orders.push(order);
        }
    }
}

pub struct TableComponent {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub eod: bool,
    pub selected_row: TableState,
    orders: OrderManager,
    table: Option<(Database, DTable)>,
    selected_column: usize,
    selection_area_corner: Option<(usize, usize)>,
    column_page_start: std::cell::Cell<usize>,
    scroll: VerticalScroll,
    key_config: KeyConfig,
}

impl TableComponent {
    pub fn new(key_config: KeyConfig) -> Self {
        Self {
            selected_row: TableState::default(),
            headers: vec![],
            rows: vec![],
            orders: OrderManager::new(),
            table: None,
            selected_column: 0,
            selection_area_corner: None,
            column_page_start: std::cell::Cell::new(0),
            scroll: VerticalScroll::new(false, false),
            eod: false,
            key_config,
        }
    }

    fn title(&self) -> String {
        self.table.as_ref().map_or(" - ".to_string(), |table| {
            format!("{}.{}", table.0.name, table.1.name)
        })
    }

    pub fn update(
        &mut self,
        rows: Vec<Vec<String>>,
        headers: Vec<String>,
        database: Database,
        table: DTable,
        hold_cusor_position: bool,
    ) {
        self.selected_row.select(None);
        if !rows.is_empty() {
            self.selected_row.select(Some(0))
        }
        self.headers = headers;
        self.rows = rows;
        self.selected_column = if hold_cusor_position {
            self.selected_column
        } else {
            0
        };
        self.selection_area_corner = None;
        self.column_page_start = std::cell::Cell::new(0);
        self.scroll = VerticalScroll::new(false, false);
        self.eod = false;
        self.table = Some((database, table));
    }

    pub fn reset(&mut self) {
        self.selected_row.select(None);
        self.headers = Vec::new();
        self.rows = Vec::new();
        self.orders = OrderManager::new();
        self.selected_column = 0;
        self.selection_area_corner = None;
        self.column_page_start = std::cell::Cell::new(0);
        self.scroll = VerticalScroll::new(false, false);
        self.eod = false;
        self.table = None;
    }

    fn reset_selection(&mut self) {
        self.selection_area_corner = None;
    }

    pub fn add_order(&mut self) {
        self.orders.add_order(self.selected_column)
    }

    pub fn generate_order_query(&mut self) -> Option<String> {
        self.orders.generate_order_query()
    }

    pub fn generate_header_icons(&mut self) -> Vec<String> {
        self.orders.generate_header_icons(self.headers.len())
    }

    pub fn end(&mut self) {
        self.eod = true;
    }

    fn next_row(&mut self, lines: usize) {
        let i = match self.selected_row.selected() {
            Some(i) => {
                if i + lines >= self.rows.len() {
                    Some(self.rows.len().saturating_sub(1))
                } else {
                    Some(i + lines)
                }
            }
            None => None,
        };
        self.reset_selection();
        self.selected_row.select(i);
    }

    fn previous_row(&mut self, lines: usize) {
        let i = match self.selected_row.selected() {
            Some(i) => {
                if i <= lines {
                    Some(0)
                } else {
                    Some(i.saturating_sub(lines))
                }
            }
            None => None,
        };
        self.reset_selection();
        self.selected_row.select(i);
    }

    fn scroll_to_top(&mut self) {
        if self.rows.is_empty() {
            return;
        }
        self.reset_selection();
        self.selected_row.select(Some(0));
    }

    fn scroll_to_bottom(&mut self) {
        if self.rows.is_empty() {
            return;
        }
        self.reset_selection();
        self.selected_row
            .select(Some(self.rows.len().saturating_sub(1)));
    }

    fn next_column(&mut self) {
        if self.rows.is_empty() {
            return;
        }
        self.reset_selection();
        if self.selected_column >= self.headers.len().saturating_sub(1) {
            return;
        }
        self.selected_column += 1;
    }

    fn previous_column(&mut self) {
        if self.rows.is_empty() {
            return;
        }
        self.reset_selection();
        if self.selected_column == 0 {
            return;
        }
        self.selected_column -= 1;
    }

    fn expand_selected_area_x(&mut self, positive: bool) {
        if self.selection_area_corner.is_none() {
            self.selection_area_corner = Some((
                self.selected_column,
                self.selected_row.selected().unwrap_or(0),
            ));
        }
        if let Some((x, y)) = self.selection_area_corner {
            self.selection_area_corner = Some((
                if positive {
                    (x + 1).min(self.headers.len().saturating_sub(1))
                } else {
                    x.saturating_sub(1)
                },
                y,
            ));
        }
    }

    fn expand_selected_area_y(&mut self, positive: bool) {
        if self.selection_area_corner.is_none() {
            self.selection_area_corner = Some((
                self.selected_column,
                self.selected_row.selected().unwrap_or(0),
            ));
        }
        if let Some((x, y)) = self.selection_area_corner {
            self.selection_area_corner = Some((
                x,
                if positive {
                    (y + 1).min(self.rows.len().saturating_sub(1))
                } else {
                    y.saturating_sub(1)
                },
            ));
        }
    }

    pub fn selected_cells(&self) -> Option<String> {
        if let Some((x, y)) = self.selection_area_corner {
            let selected_row_index = self.selected_row.selected()?;
            return Some(
                self.rows[y.min(selected_row_index)..y.max(selected_row_index) + 1]
                    .iter()
                    .map(|row| {
                        row[x.min(self.selected_column)..x.max(self.selected_column) + 1].join(",")
                    })
                    .collect::<Vec<String>>()
                    .join("\n"),
            );
        }
        self.rows
            .get(self.selected_row.selected()?)?
            .get(self.selected_column)
            .map(|cell| cell.to_string())
    }

    fn selected_column_index(&self) -> usize {
        if let Some((x, _)) = self.selection_area_corner {
            return x;
        }
        self.selected_column
    }

    fn is_selected_cell(
        &self,
        row_index: usize,
        column_index: usize,
        selected_column_index: usize,
    ) -> bool {
        if let Some((x, y)) = self.selection_area_corner {
            let x_in_page = x
                .saturating_add(1)
                .saturating_sub(self.column_page_start.get());
            return matches!(
                self.selected_row.selected(),
                Some(selected_row_index)
                if (x_in_page.min(selected_column_index).max(1)..x_in_page.max(selected_column_index) + 1)
                    .contains(&column_index)
                    && (y.min(selected_row_index)..y.max(selected_row_index) + 1)
                        .contains(&row_index)
            );
        }
        matches!(
            self.selected_row.selected(),
            Some(selected_row_index) if row_index == selected_row_index &&  column_index == selected_column_index
        )
    }

    fn is_number_column(&self, row_index: usize, column_index: usize) -> bool {
        matches!(
            self.selected_row.selected(),
            Some(selected_row_index) if row_index == selected_row_index && 0 == column_index
        )
    }

    fn headers(&self, left: usize, right: usize) -> Vec<String> {
        let mut headers = self.headers.clone()[left..right].to_vec();
        headers.insert(0, "".to_string());
        headers
    }

    fn rows(&self, left: usize, right: usize) -> Vec<Vec<String>> {
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

    fn calculate_cell_widths(
        &self,
        area_width: u16,
    ) -> (usize, Vec<String>, Vec<Vec<String>>, Vec<Constraint>) {
        if self.rows.is_empty() {
            return (0, Vec::new(), Vec::new(), Vec::new());
        }
        if self.selected_column_index() < self.column_page_start.get() {
            self.column_page_start.set(self.selected_column_index());
        }

        let far_right_column_index = self.selected_column_index();
        let mut column_index = self.selected_column_index();
        let number_column_width = (self.rows.len() + 1).to_string().width() as u16;
        let mut widths = Vec::new();
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
            if widths.iter().map(|(_, width)| width).sum::<usize>() + length + widths.len() + 1
                >= area_width.saturating_sub(number_column_width) as usize
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
        widths.reverse();

        let far_left_column_index = column_index;
        let selected_column_index = widths.len().saturating_sub(1);
        let mut column_index = far_right_column_index + 1;
        while widths.iter().map(|(_, width)| width).sum::<usize>() + widths.len()
            < area_width.saturating_sub(number_column_width) as usize
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
        if self.selected_column_index() != self.headers.len().saturating_sub(1)
            && column_index.saturating_sub(1) != self.headers.len().saturating_sub(1)
        {
            widths.pop();
        }
        let far_right_column_index = column_index;
        let mut constraints = widths
            .iter()
            .map(|(_, width)| Constraint::Length(*width as u16))
            .collect::<Vec<Constraint>>();
        if self.selected_column_index() != self.headers.len().saturating_sub(1)
            && column_index.saturating_sub(1) != self.headers.len().saturating_sub(1)
        {
            constraints.push(Constraint::Min(10));
        }
        constraints.insert(0, Constraint::Length(number_column_width));
        self.column_page_start.set(far_left_column_index);

        (
            self.selection_area_corner
                .map_or(selected_column_index + 1, |(x, _)| {
                    if x > self.selected_column {
                        (selected_column_index + 1)
                            .saturating_sub(x.saturating_sub(self.selected_column))
                    } else {
                        (selected_column_index + 1)
                            .saturating_add(self.selected_column.saturating_sub(x))
                    }
                }),
            self.headers(far_left_column_index, far_right_column_index),
            self.rows(far_left_column_index, far_right_column_index),
            constraints,
        )
    }
}

impl StatefulDrawableComponent for TableComponent {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect, focused: bool) -> Result<()> {
        let chunks = Layout::default()
            .vertical_margin(1)
            .horizontal_margin(1)
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(2),
                    Constraint::Min(1),
                    Constraint::Length(2),
                ]
                .as_ref(),
            )
            .split(area);

        f.render_widget(
            Block::default()
                .title(self.title())
                .borders(Borders::ALL)
                .style(if focused {
                    Style::default()
                } else {
                    Style::default().fg(Color::DarkGray)
                }),
            area,
        );

        self.selected_row.selected().map_or_else(
            || {
                self.scroll.reset();
            },
            |selection| {
                self.scroll.update(
                    selection,
                    self.rows.len(),
                    chunks[1].height.saturating_sub(2) as usize,
                );
            },
        );

        let block = Block::default().borders(Borders::NONE);
        let (selected_column_index, headers, rows, constraints) =
            self.calculate_cell_widths(block.inner(chunks[0]).width);
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
                    } else if self.is_number_column(row_index, column_index) {
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
            .style(if focused {
                Style::default()
            } else {
                Style::default().fg(Color::DarkGray)
            })
            .widths(&constraints);
        let mut state = self.selected_row.clone();
        f.render_stateful_widget(
            table,
            chunks[1],
            if let Some((_, y)) = self.selection_area_corner {
                state.select(Some(y));
                &mut state
            } else {
                &mut self.selected_row
            },
        );

        TableValueComponent::new(self.selected_cells().unwrap_or_default())
            .draw(f, chunks[0], focused)?;

        TableStatusComponent::new(
            if self.rows.is_empty() {
                None
            } else {
                Some(self.rows.len())
            },
            if self.headers.is_empty() {
                None
            } else {
                Some(self.headers.len())
            },
            self.table.as_ref().map(|t| t.1.clone()),
        )
        .draw(f, chunks[2], focused)?;

        self.scroll.draw(f, chunks[1]);
        Ok(())
    }
}

impl Component for TableComponent {
    fn commands(&self, out: &mut Vec<CommandInfo>) {
        out.push(CommandInfo::new(command::extend_selection_by_one_cell(
            &self.key_config,
        )));
        out.push(CommandInfo::new(command::sort_by_column(&self.key_config)));
    }

    fn event(&mut self, key: Key) -> Result<EventState> {
        if key == self.key_config.scroll_left {
            self.previous_column();
            return Ok(EventState::Consumed);
        } else if key == self.key_config.scroll_down {
            self.next_row(1);
            return Ok(EventState::NotConsumed);
        } else if key == self.key_config.scroll_down_multiple_lines {
            self.next_row(10);
            return Ok(EventState::NotConsumed);
        } else if key == self.key_config.scroll_up {
            self.previous_row(1);
            return Ok(EventState::Consumed);
        } else if key == self.key_config.scroll_up_multiple_lines {
            self.previous_row(10);
            return Ok(EventState::Consumed);
        } else if key == self.key_config.scroll_to_top {
            self.scroll_to_top();
            return Ok(EventState::Consumed);
        } else if key == self.key_config.scroll_to_bottom {
            self.scroll_to_bottom();
            return Ok(EventState::Consumed);
        } else if key == self.key_config.scroll_right {
            self.next_column();
            return Ok(EventState::Consumed);
        } else if key == self.key_config.extend_selection_by_one_cell_left {
            self.expand_selected_area_x(false);
            return Ok(EventState::Consumed);
        } else if key == self.key_config.extend_selection_by_one_cell_up {
            self.expand_selected_area_y(false);
            return Ok(EventState::Consumed);
        } else if key == self.key_config.extend_selection_by_one_cell_down {
            self.expand_selected_area_y(true);
            return Ok(EventState::Consumed);
        } else if key == self.key_config.extend_selection_by_one_cell_right {
            self.expand_selected_area_x(true);
            return Ok(EventState::Consumed);
        }
        Ok(EventState::NotConsumed)
    }
}

#[cfg(test)]
mod test {
    use super::{KeyConfig, Order, OrderManager, TableComponent};
    use tui::layout::Constraint;

    #[test]
    fn test_headers() {
        let mut component = TableComponent::new(KeyConfig::default());
        component.headers = vec!["a", "b", "c"].iter().map(|h| h.to_string()).collect();
        assert_eq!(component.headers(1, 2), vec!["", "b"])
    }

    #[test]
    fn test_rows() {
        let mut component = TableComponent::new(KeyConfig::default());
        component.rows = vec![
            vec!["a", "b", "c"].iter().map(|h| h.to_string()).collect(),
            vec!["d", "e", "f"].iter().map(|h| h.to_string()).collect(),
        ];
        assert_eq!(component.rows(1, 2), vec![vec!["1", "b"], vec!["2", "e"]],)
    }

    #[test]
    fn test_query() {
        let asc_order = Order::new(1, true);
        let desc_order = Order::new(2, false);

        assert_eq!(asc_order.query(), "1 ASC".to_string());
        assert_eq!(desc_order.query(), "2 DESC".to_string());
    }

    #[test]
    fn test_generate_order_query() {
        let mut order_manager = OrderManager::new();

        // If orders is empty, it should return None.
        assert_eq!(order_manager.generate_order_query(), None);

        order_manager.add_order(1);
        order_manager.add_order(1);
        order_manager.add_order(2);
        assert_eq!(
            order_manager.generate_order_query(),
            Some("ORDER BY 2 DESC, 3 ASC".to_string())
        )
    }

    #[test]
    fn test_generate_header_icons() {
        let mut order_manager = OrderManager::new();
        assert_eq!(order_manager.generate_header_icons(1), vec![String::new()]);

        order_manager.add_order(1);
        order_manager.add_order(1);
        order_manager.add_order(2);
        assert_eq!(
            order_manager.generate_header_icons(3),
            vec![String::new(), "↓1".to_string(), "↑2".to_string()]
        );
        assert_eq!(
            order_manager.generate_header_icons(4),
            vec![
                String::new(),
                "↓1".to_string(),
                "↑2".to_string(),
                String::new()
            ]
        );
    }

    #[test]
    fn test_add_order() {
        let mut order_manager = OrderManager::new();

        // press first time, condition is asc.
        order_manager.add_order(1);
        assert_eq!(order_manager.orders, vec![Order::new(2, true)]);

        // press twice times, condition is desc.
        order_manager.add_order(1);
        assert_eq!(order_manager.orders, vec![Order::new(2, false)]);

        // press another column, this column is second order.
        order_manager.add_order(2);
        assert_eq!(
            order_manager.orders,
            vec![Order::new(2, false), Order::new(3, true)]
        );

        // press three times, removed.
        order_manager.add_order(1);
        assert_eq!(order_manager.orders, vec![Order::new(3, true)]);
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

        let mut component = TableComponent::new(KeyConfig::default());
        component.headers = vec!["1", "2", "3"].iter().map(|h| h.to_string()).collect();
        component.rows = vec![
            vec!["a", "b", "c"].iter().map(|h| h.to_string()).collect(),
            vec!["d", "e", "f"].iter().map(|h| h.to_string()).collect(),
        ];
        component.selected_row.select(Some(1));
        component.selected_column = 1;
        component.expand_selected_area_x(false);
        assert_eq!(component.selection_area_corner, Some((0, 1)));
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

        let mut component = TableComponent::new(KeyConfig::default());
        component.headers = vec!["1", "2", "3"].iter().map(|h| h.to_string()).collect();
        component.rows = vec![
            vec!["a", "b", "c"].iter().map(|h| h.to_string()).collect(),
            vec!["d", "e", "f"].iter().map(|h| h.to_string()).collect(),
        ];
        component.selected_row.select(Some(1));
        component.selected_column = 1;
        component.expand_selected_area_x(true);
        assert_eq!(component.selection_area_corner, Some((2, 1)));
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

        let mut component = TableComponent::new(KeyConfig::default());
        component.rows = vec![
            vec!["a", "b", "c"].iter().map(|h| h.to_string()).collect(),
            vec!["d", "e", "f"].iter().map(|h| h.to_string()).collect(),
        ];
        component.selected_row.select(Some(1));
        component.selected_column = 1;
        component.expand_selected_area_y(false);
        assert_eq!(component.selection_area_corner, Some((1, 0)));
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

        let mut component = TableComponent::new(KeyConfig::default());
        component.rows = vec![
            vec!["a", "b", "c"].iter().map(|h| h.to_string()).collect(),
            vec!["d", "e", "f"].iter().map(|h| h.to_string()).collect(),
        ];
        component.selected_row.select(Some(0));
        component.selected_column = 1;
        component.expand_selected_area_y(true);
        assert_eq!(component.selection_area_corner, Some((1, 1)));
        assert_eq!(component.selected_cells(), Some("b\ne".to_string()));
    }

    #[test]
    fn test_is_number_column() {
        let mut component = TableComponent::new(KeyConfig::default());
        component.headers = vec!["1", "2", "3"].iter().map(|h| h.to_string()).collect();
        component.rows = vec![
            vec!["a", "b", "c"].iter().map(|h| h.to_string()).collect(),
            vec!["d", "e", "f"].iter().map(|h| h.to_string()).collect(),
        ];
        component.selected_row.select(Some(0));
        assert!(component.is_number_column(0, 0));
        assert!(!component.is_number_column(0, 1));
    }

    #[test]
    fn test_selected_cell_when_one_cell_selected() {
        //    1  2 3
        // 1 |a| b c
        // 2  d  e f

        let mut component = TableComponent::new(KeyConfig::default());
        component.headers = vec!["1", "2", "3"].iter().map(|h| h.to_string()).collect();
        component.rows = vec![
            vec!["a", "b", "c"].iter().map(|h| h.to_string()).collect(),
            vec!["d", "e", "f"].iter().map(|h| h.to_string()).collect(),
        ];
        component.selected_row.select(Some(0));
        assert_eq!(component.selected_cells(), Some("a".to_string()));
    }

    #[test]
    fn test_selected_cell_when_multiple_cells_selected() {
        //    1  2  3
        // 1 |a  b| c
        // 2 |d  e| f

        let mut component = TableComponent::new(KeyConfig::default());
        component.headers = vec!["1", "2", "3"].iter().map(|h| h.to_string()).collect();
        component.rows = vec![
            vec!["a", "b", "c"].iter().map(|h| h.to_string()).collect(),
            vec!["d", "e", "f"].iter().map(|h| h.to_string()).collect(),
        ];
        component.selected_row.select(Some(0));
        component.selection_area_corner = Some((1, 1));
        assert_eq!(component.selected_cells(), Some("a,b\nd,e".to_string()));
    }

    #[test]
    fn test_is_selected_cell_when_one_cell_selected() {
        //    1  2 3
        // 1 |a| b c
        // 2  d  e f

        let mut component = TableComponent::new(KeyConfig::default());
        component.headers = vec!["1", "2", "3"].iter().map(|h| h.to_string()).collect();
        component.rows = vec![
            vec!["a", "b", "c"].iter().map(|h| h.to_string()).collect(),
            vec!["d", "e", "f"].iter().map(|h| h.to_string()).collect(),
        ];
        component.selected_row.select(Some(0));
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

        let mut component = TableComponent::new(KeyConfig::default());
        component.headers = vec!["1", "2", "3"].iter().map(|h| h.to_string()).collect();
        component.rows = vec![
            vec!["a", "b", "c"].iter().map(|h| h.to_string()).collect(),
            vec!["d", "e", "f"].iter().map(|h| h.to_string()).collect(),
        ];
        component.selected_row.select(Some(0));
        component.selection_area_corner = Some((1, 1));
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
    fn test_calculate_cell_widths_when_sum_of_cell_widths_is_greater_than_table_width() {
        let mut component = TableComponent::new(KeyConfig::default());
        component.headers = vec!["1", "2", "3"].iter().map(|h| h.to_string()).collect();
        component.rows = vec![
            vec!["aaaaa", "bbbbb", "ccccc"]
                .iter()
                .map(|h| h.to_string())
                .collect(),
            vec!["d", "e", "f"].iter().map(|h| h.to_string()).collect(),
        ];
        let (selected_column_index, headers, rows, constraints) =
            component.calculate_cell_widths(10);
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
    }

    #[test]
    fn test_calculate_cell_widths_when_sum_of_cell_widths_is_less_than_table_width() {
        let mut component = TableComponent::new(KeyConfig::default());
        component.headers = vec!["1", "2", "3"].iter().map(|h| h.to_string()).collect();
        component.rows = vec![
            vec!["aaaaa", "bbbbb", "ccccc"]
                .iter()
                .map(|h| h.to_string())
                .collect(),
            vec!["d", "e", "f"].iter().map(|h| h.to_string()).collect(),
        ];

        let (selected_column_index, headers, rows, constraints) =
            component.calculate_cell_widths(20);
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
                Constraint::Length(5),
            ]
        );
    }

    #[test]
    fn test_calculate_cell_widths_when_component_has_multiple_rows() {
        let mut component = TableComponent::new(KeyConfig::default());
        component.headers = vec!["1", "2", "3"].iter().map(|h| h.to_string()).collect();
        component.rows = vec![
            vec!["aaaaa", "bbbbb", "ccccc"]
                .iter()
                .map(|h| h.to_string())
                .collect(),
            vec!["dddddddddd", "e", "f"]
                .iter()
                .map(|h| h.to_string())
                .collect(),
        ];

        let (selected_column_index, headers, rows, constraints) =
            component.calculate_cell_widths(20);
        assert_eq!(selected_column_index, 1);
        assert_eq!(headers, vec!["", "1", "2", "3"]);
        assert_eq!(
            rows,
            vec![
                vec!["1", "aaaaa", "bbbbb", "ccccc"],
                vec!["2", "dddddddddd", "e", "f"]
            ]
        );
        assert_eq!(
            constraints,
            vec![
                Constraint::Length(1),
                Constraint::Length(10),
                Constraint::Length(5),
                Constraint::Length(5),
            ]
        );
    }
}
