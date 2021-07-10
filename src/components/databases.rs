use super::{utils::scroll_vertical::VerticalScroll, Component, DrawableComponent};
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
    widgets::{Block, Borders},
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
    pub tree: DatabaseTree,
    pub filterd_tree: Option<DatabaseTree>,
    pub scroll: VerticalScroll,
    pub input: String,
    pub input_cursor_x: u16,
    pub focus_block: FocusBlock,
}

impl DatabasesComponent {
    pub fn new() -> Self {
        Self {
            tree: DatabaseTree::default(),
            filterd_tree: None,
            scroll: VerticalScroll::new(),
            input: String::new(),
            input_cursor_x: 0,
            focus_block: FocusBlock::Tree,
        }
    }

    pub fn update(&mut self, list: &[Database], collapsed: &BTreeSet<&String>) -> Result<()> {
        self.tree = DatabaseTree::new(list, collapsed)?;
        self.filterd_tree = None;
        self.input = String::new();
        self.input_cursor_x = 0;
        Ok(())
    }

    pub fn tree_focused(&self) -> bool {
        matches!(self.focus_block, FocusBlock::Tree)
    }

    pub fn tree(&self) -> &DatabaseTree {
        self.filterd_tree.as_ref().unwrap_or(&self.tree)
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
        let tree_height = usize::from(area.height.saturating_sub(4));
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
                self.scroll.update(
                    selection.index,
                    selection.count.saturating_sub(2),
                    tree_height,
                );
            },
        );

        let mut items = tree
            .iterate(self.scroll.get_top(), tree_height)
            .map(|(item, selected)| Self::tree_item_to_span(item.clone(), selected, area.width))
            .collect::<Vec<Span>>();

        items.insert(
            0,
            Span::styled(
                format!(
                    "{}",
                    (0..area.width as usize)
                        .map(|_| HORIZONTAL)
                        .collect::<Vec<&str>>()
                        .join("")
                ),
                Style::default(),
            ),
        );
        items.insert(
            0,
            Span::styled(
                format!(
                    "{}{:w$}",
                    if self.input.is_empty() && matches!(self.focus_block, FocusBlock::Tree) {
                        "Press / to filter".to_string()
                    } else {
                        self.input.clone()
                    },
                    w = area.width as usize
                ),
                if let FocusBlock::Filter = self.focus_block {
                    Style::default()
                } else {
                    Style::default().fg(Color::DarkGray)
                },
            ),
        );

        let title = "Databases";
        draw_list_block(
            f,
            area,
            Block::default()
                .title(Span::styled(title, Style::default()))
                .style(if focused {
                    Style::default()
                } else {
                    Style::default().fg(Color::DarkGray)
                })
                .borders(Borders::ALL)
                .border_style(Style::default()),
            items.into_iter(),
        );
        self.scroll.draw(f, area);
        if let FocusBlock::Filter = self.focus_block {
            f.set_cursor(
                area.x + self.input.width() as u16 + 1 - self.input_cursor_x,
                area.y + 1,
            )
        }
    }
}

impl DrawableComponent for DatabasesComponent {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect, focused: bool) -> Result<()> {
        if true {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(area);

            self.draw_tree(f, chunks[0], focused);
        }
        Ok(())
    }
}

impl Component for DatabasesComponent {
    fn event(&mut self, key: Key) -> Result<()> {
        match key {
            Key::Char('/') if matches!(self.focus_block, FocusBlock::Tree) => {
                self.focus_block = FocusBlock::Filter
            }
            Key::Char(c) if matches!(self.focus_block, FocusBlock::Filter) => {
                self.input.push(c);
                self.filterd_tree = Some(self.tree.filter(self.input.clone()))
            }
            Key::Delete | Key::Backspace => {
                if self.input.is_empty() {
                    self.filterd_tree = None
                } else {
                    if self.input_cursor_x == 0 {
                        self.input.pop();
                        return Ok(());
                    }
                    if self.input.width() - self.input_cursor_x as usize > 0 {
                        self.input.remove(
                            self.input
                                .width()
                                .saturating_sub(self.input_cursor_x as usize)
                                .saturating_sub(1),
                        );
                    }
                    self.filterd_tree = Some(self.tree.filter(self.input.clone()))
                }
            }
            Key::Left => self.decrement_input_cursor_x(),
            Key::Right => self.increment_input_cursor_x(),
            Key::Enter if matches!(self.focus_block, FocusBlock::Filter) => {
                self.focus_block = FocusBlock::Tree
            }
            key => tree_nav(
                if let Some(tree) = self.filterd_tree.as_mut() {
                    tree
                } else {
                    &mut self.tree
                },
                key,
            ),
        }
        Ok(())
    }
}

fn tree_nav(tree: &mut DatabaseTree, key: Key) {
    if let Some(common_nav) = common_nav(key) {
        tree.move_selection(common_nav);
    } else {
        false;
    }
}
