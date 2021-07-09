use super::{utils::vertical_scroll::VerticalScroll, Component, DrawableComponent};
use crate::event::Key;
use crate::ui::common_nav;
use crate::ui::scrolllist::draw_list_block;
use anyhow::Result;
use database_tree::{DatabaseTree, DatabaseTreeItem};
use std::convert::From;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders},
    Frame,
};

// ▸
const FOLDER_ICON_COLLAPSED: &str = "\u{25b8}";
// ▾
const FOLDER_ICON_EXPANDED: &str = "\u{25be}";
const EMPTY_STR: &str = "";

pub struct DatabasesComponent {
    pub tree: DatabaseTree,
    pub scroll: VerticalScroll,
}

impl DatabasesComponent {
    pub fn new() -> Self {
        Self {
            tree: DatabaseTree::default(),
            scroll: VerticalScroll::new(),
        }
    }

    fn tree_item_to_span(item: &DatabaseTreeItem, selected: bool, width: u16) -> Span<'_> {
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
        let tree_height = usize::from(area.height.saturating_sub(2));
        self.tree.visual_selection().map_or_else(
            || {
                self.scroll.reset();
            },
            |selection| {
                self.scroll
                    .update(selection.index, selection.count, tree_height);
            },
        );

        let items = self
            .tree
            .iterate(self.scroll.get_top(), tree_height)
            .map(|(item, selected)| Self::tree_item_to_span(item, selected, area.width));

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
            items,
        );
        self.scroll.draw(f, area);
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
        if tree_nav(&mut self.tree, key) {
            return Ok(());
        }
        Ok(())
    }
}

fn tree_nav(tree: &mut DatabaseTree, key: Key) -> bool {
    if let Some(common_nav) = common_nav(key) {
        tree.move_selection(common_nav)
    } else {
        false
    }
}
