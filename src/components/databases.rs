use super::{
    compute_character_width, utils::scroll_vertical::VerticalScroll, Component, DrawableComponent,
    EventState,
};
use crate::components::command::CommandInfo;
use crate::event::Key;
use crate::ui::common_nav;
use crate::ui::scrolllist::draw_list_block;
use anyhow::Result;
use database_tree::{Database, DatabaseTree, DatabaseTreeItem};
use std::collections::BTreeSet;
use std::convert::From;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    symbols::line::HORIZONTAL,
    text::Span,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use unicode_width::UnicodeWidthStr;

// ▸
const FOLDER_ICON_COLLAPSED: &str = "\u{25b8}";
// ▾
const FOLDER_ICON_EXPANDED: &str = "\u{25be}";
const EMPTY_STR: &str = "";

pub enum FocusBlock {
    Filter,
    Tree,
}

pub struct DatabasesComponent {
    tree: DatabaseTree,
    filterd_tree: Option<DatabaseTree>,
    scroll: VerticalScroll,
    input: Vec<char>,
    input_idx: usize,
    input_cursor_position: u16,
    focus_block: FocusBlock,
}

impl DatabasesComponent {
    pub fn new() -> Self {
        Self {
            tree: DatabaseTree::default(),
            filterd_tree: None,
            scroll: VerticalScroll::new(),
            input: Vec::new(),
            input_idx: 0,
            input_cursor_position: 0,
            focus_block: FocusBlock::Tree,
        }
    }

    fn input_str(&self) -> String {
        self.input.iter().collect()
    }

    pub fn update(&mut self, list: &[Database]) -> Result<()> {
        self.tree = DatabaseTree::new(list, &BTreeSet::new())?;
        self.filterd_tree = None;
        self.input = Vec::new();
        self.input_idx = 0;
        Ok(())
    }

    pub fn tree_focused(&self) -> bool {
        matches!(self.focus_block, FocusBlock::Tree)
    }

    pub fn tree(&self) -> &DatabaseTree {
        self.filterd_tree.as_ref().unwrap_or(&self.tree)
    }

    fn tree_item_to_span(item: DatabaseTreeItem, selected: bool, width: u16) -> Span<'static> {
        let name = item.kind().name();
        let indent = item.info().indent();

        let indent_str = if indent == 0 {
            String::from("")
        } else {
            format!("{:w$}", " ", w = (indent as usize) * 2)
        };

        let is_database = item.kind().is_database();
        let path_arrow = if is_database {
            if item.kind().is_database_collapsed() {
                FOLDER_ICON_COLLAPSED
            } else {
                FOLDER_ICON_EXPANDED
            }
        } else {
            EMPTY_STR
        };

        let name = format!(
            "{}{}{:w$}",
            indent_str,
            path_arrow,
            name,
            w = width as usize
        );
        Span::styled(
            name,
            if selected {
                Style::default().bg(Color::Blue)
            } else {
                Style::default()
            },
        )
    }

    fn draw_tree<B: Backend>(&self, f: &mut Frame<B>, area: Rect, focused: bool) {
        f.render_widget(
            Block::default()
                .title("Databases")
                .borders(Borders::ALL)
                .style(if focused {
                    Style::default()
                } else {
                    Style::default().fg(Color::DarkGray)
                }),
            area,
        );

        let chunks = Layout::default()
            .vertical_margin(1)
            .horizontal_margin(1)
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Min(1)].as_ref())
            .split(area);

        let filter = Paragraph::new(Span::styled(
            format!(
                "{}{:w$}",
                if self.input.is_empty() && matches!(self.focus_block, FocusBlock::Tree) {
                    "Filter tables".to_string()
                } else {
                    self.input_str()
                },
                w = area.width as usize
            ),
            if let FocusBlock::Filter = self.focus_block {
                Style::default()
            } else {
                Style::default().fg(Color::DarkGray)
            },
        ))
        .block(Block::default().borders(Borders::BOTTOM));
        f.render_widget(filter, chunks[0]);

        let tree_height = chunks[1].height as usize;
        let tree = if let Some(tree) = self.filterd_tree.as_ref() {
            tree
        } else {
            &self.tree
        };
        tree.visual_selection().map_or_else(
            || {
                self.scroll.reset();
            },
            |selection| {
                self.scroll
                    .update(selection.index, selection.count, tree_height);
            },
        );

        let items = tree
            .iterate(self.scroll.get_top(), tree_height)
            .map(|(item, selected)| Self::tree_item_to_span(item.clone(), selected, area.width))
            .collect::<Vec<Span>>();

        draw_list_block(
            f,
            chunks[1],
            Block::default().borders(Borders::NONE),
            items.into_iter(),
        );
        self.scroll.draw(f, area);
        if let FocusBlock::Filter = self.focus_block {
            f.set_cursor(area.x + self.input_cursor_position + 1, area.y + 1)
        }
    }
}

impl DrawableComponent for DatabasesComponent {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect, focused: bool) -> Result<()> {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(100)].as_ref())
            .split(area);

        self.draw_tree(f, chunks[0], focused);
        Ok(())
    }
}

impl Component for DatabasesComponent {
    fn commands(&self, out: &mut Vec<CommandInfo>) {}

    fn event(&mut self, key: Key) -> Result<EventState> {
        let input_str: String = self.input.iter().collect();
        if tree_nav(
            if let Some(tree) = self.filterd_tree.as_mut() {
                tree
            } else {
                &mut self.tree
            },
            key,
        ) {
            return Ok(EventState::Consumed);
        }
        match key {
            Key::Char('/') if matches!(self.focus_block, FocusBlock::Tree) => {
                self.focus_block = FocusBlock::Filter;
                return Ok(EventState::Consumed);
            }
            Key::Char(c) if matches!(self.focus_block, FocusBlock::Filter) => {
                self.input.insert(self.input_idx, c);
                self.input_idx += 1;
                self.input_cursor_position += compute_character_width(c);

                self.filterd_tree = Some(self.tree.filter(self.input_str()));
                return Ok(EventState::Consumed);
            }
            Key::Delete | Key::Backspace if matches!(self.focus_block, FocusBlock::Filter) => {
                if input_str.width() > 0 {
                    if !self.input.is_empty() && self.input_idx > 0 {
                        let last_c = self.input.remove(self.input_idx - 1);
                        self.input_idx -= 1;
                        self.input_cursor_position -= compute_character_width(last_c);
                    }

                    self.filterd_tree = if self.input.is_empty() {
                        None
                    } else {
                        Some(self.tree.filter(self.input_str()))
                    };
                    return Ok(EventState::Consumed);
                }
            }
            Key::Left if matches!(self.focus_block, FocusBlock::Filter) => {
                if !self.input.is_empty() && self.input_idx > 0 {
                    self.input_idx -= 1;
                    self.input_cursor_position = self
                        .input_cursor_position
                        .saturating_sub(compute_character_width(self.input[self.input_idx]));
                }
                return Ok(EventState::Consumed);
            }
            Key::Ctrl('a') => {
                if !self.input.is_empty() && self.input_idx > 0 {
                    self.input_idx = 0;
                    self.input_cursor_position = 0
                }
                return Ok(EventState::Consumed);
            }
            Key::Right if matches!(self.focus_block, FocusBlock::Filter) => {
                if self.input_idx < self.input.len() {
                    let next_c = self.input[self.input_idx];
                    self.input_idx += 1;
                    self.input_cursor_position += compute_character_width(next_c);
                }
                return Ok(EventState::Consumed);
            }
            Key::Ctrl('e') => {
                if self.input_idx < self.input.len() {
                    self.input_idx = self.input.len();
                    self.input_cursor_position = self.input_str().width() as u16;
                }
                return Ok(EventState::Consumed);
            }
            Key::Enter if matches!(self.focus_block, FocusBlock::Filter) => {
                self.focus_block = FocusBlock::Tree;
                return Ok(EventState::Consumed);
            }
            _ => (),
        }
        Ok(EventState::NotConsumed)
    }
}

fn tree_nav(tree: &mut DatabaseTree, key: Key) -> bool {
    if let Some(common_nav) = common_nav(key) {
        tree.move_selection(common_nav)
    } else {
        false
    }
}
