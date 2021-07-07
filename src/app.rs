use crate::components::utils::scroll_vertical::VerticalScroll;
use crate::{
    components::DatabasesComponent,
    user_config::{Connection, UserConfig},
};
use sqlx::mysql::MySqlPool;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Cell, ListState, Row, Table as WTable, TableState},
    Frame,
};
use unicode_width::UnicodeWidthStr;

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

pub enum FocusBlock {
    DabataseList,
    RecordTable,
    ConnectionList,
    Query,
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct Column {
    #[sqlx(rename = "Field")]
    pub field: String,
    #[sqlx(rename = "Type")]
    pub r#type: String,
    #[sqlx(rename = "Collation")]
    pub collation: String,
    #[sqlx(rename = "Null")]
    pub null: String,
}

pub struct RecordTable {
    pub state: TableState,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub column_index: usize,
    pub scroll: VerticalScroll,
}

impl Default for RecordTable {
    fn default() -> Self {
        Self {
            state: TableState::default(),
            headers: vec![],
            rows: vec![],
            column_index: 0,
            scroll: VerticalScroll::new(),
        }
    }
}

impl RecordTable {
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.rows.len() - 1 {
                    Some(i)
                } else {
                    Some(i + 1)
                }
            }
            None => None,
        };
        self.state.select(i);
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    Some(i)
                } else {
                    Some(i - 1)
                }
            }
            None => None,
        };
        self.state.select(i);
    }

    pub fn next_column(&mut self) {
        if self.headers.len() > 9 && self.column_index < self.headers.len() - 9 {
            self.column_index += 1
        }
    }

    pub fn previous_column(&mut self) {
        if self.column_index > 0 {
            self.column_index -= 1
        }
    }

    pub fn headers(&self) -> Vec<String> {
        let mut headers = self.headers[self.column_index..].to_vec();
        headers.insert(0, "".to_string());
        headers
    }

    pub fn rows(&self) -> Vec<Vec<String>> {
        let mut rows = self
            .rows
            .iter()
            .map(|row| row[self.column_index..].to_vec())
            .collect::<Vec<Vec<String>>>();
        for (index, row) in rows.iter_mut().enumerate() {
            row.insert(0, (index + 1).to_string())
        }
        rows
    }

    pub fn draw<B: Backend>(
        &mut self,
        f: &mut Frame<'_, B>,
        layout_chunk: Rect,
        focused: bool,
    ) -> anyhow::Result<()> {
        self.state.selected().map_or_else(
            || {
                self.scroll.reset();
            },
            |selection| {
                self.scroll.update(
                    selection,
                    self.rows.len(),
                    layout_chunk.height.saturating_sub(2) as usize,
                );
            },
        );

        let headers = self.headers();
        let header_cells = headers
            .iter()
            .map(|h| Cell::from(h.to_string()).style(Style::default()));
        let header = Row::new(header_cells).height(1).bottom_margin(1);
        let rows = self.rows();
        let rows = rows.iter().map(|item| {
            let height = item
                .iter()
                .map(|content| content.chars().filter(|c| *c == '\n').count())
                .max()
                .unwrap_or(0)
                + 1;
            let cells = item
                .iter()
                .map(|c| Cell::from(c.to_string()).style(Style::default()));
            Row::new(cells).height(height as u16).bottom_margin(1)
        });
        let widths = (0..10)
            .map(|_| Constraint::Percentage(10))
            .collect::<Vec<Constraint>>();
        let t = WTable::new(rows)
            .header(header)
            .block(Block::default().borders(Borders::ALL).title("Records"))
            .highlight_style(Style::default().fg(Color::Green))
            .style(if focused {
                Style::default()
            } else {
                Style::default().fg(Color::DarkGray)
            })
            .widths(&widths);
        f.render_stateful_widget(t, layout_chunk, &mut self.state);

        self.scroll.draw(f, layout_chunk);
        Ok(())
    }
}

pub struct App {
    pub input: String,
    pub input_cursor_x: u16,
    pub query: String,
    pub record_table: RecordTable,
    pub structure_table: RecordTable,
    pub focus_block: FocusBlock,
    pub selected_tab: Tab,
    pub user_config: Option<UserConfig>,
    pub selected_connection: ListState,
    pub selected_database: ListState,
    pub selected_table: ListState,
    pub databases: DatabasesComponent,
    pub pool: Option<MySqlPool>,
    pub error: Option<String>,
}

impl Default for App {
    fn default() -> App {
        App {
            input: String::new(),
            input_cursor_x: 0,
            query: String::new(),
            record_table: RecordTable::default(),
            structure_table: RecordTable::default(),
            focus_block: FocusBlock::DabataseList,
            selected_tab: Tab::Records,
            user_config: None,
            selected_connection: ListState::default(),
            selected_database: ListState::default(),
            selected_table: ListState::default(),
            databases: DatabasesComponent::new(),
            pool: None,
            error: None,
        }
    }
}

impl App {
    pub fn next_connection(&mut self) {
        if let Some(config) = &self.user_config {
            let i = match self.selected_connection.selected() {
                Some(i) => {
                    if i >= config.conn.len() - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            self.selected_connection.select(Some(i));
        }
    }

    pub fn previous_connection(&mut self) {
        if let Some(config) = &self.user_config {
            let i = match self.selected_connection.selected() {
                Some(i) => {
                    if i == 0 {
                        config.conn.len() - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.selected_connection.select(Some(i));
        }
    }

    pub fn increment_input_cursor_x(&mut self) {
        if self.input_cursor_x > 0 {
            self.input_cursor_x -= 1;
        }
    }

    pub fn decrement_input_cursor_x(&mut self) {
        if self.input_cursor_x < self.input.width() as u16 {
            self.input_cursor_x += 1;
        }
    }

    pub fn selected_connection(&self) -> Option<&Connection> {
        match &self.user_config {
            Some(config) => match self.selected_connection.selected() {
                Some(i) => config.conn.get(i),
                None => None,
            },
            None => None,
        }
    }

    pub fn table_status(&self) -> Vec<String> {
        if let Some((table, _)) = self.databases.tree.selected_table() {
            return vec![
                format!("created: {}", table.create_time.to_string()),
                format!(
                    "updated: {}",
                    table
                        .update_time
                        .map(|time| time.to_string())
                        .unwrap_or_default()
                ),
                format!(
                    "engine: {}",
                    table
                        .engine
                        .as_ref()
                        .map(|engine| engine.to_string())
                        .unwrap_or_default()
                ),
                format!("rows: {}", self.record_table.rows.len()),
            ];
        }
        Vec::new()
    }
}
