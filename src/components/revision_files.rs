use super::{
    utils::scroll_vertical::VerticalScroll, CommandBlocking, CommandInfo, Component,
    DrawableComponent, EventState,
};
use crate::event::{Event as Ev, Key};
use crate::ui::common_nav;
use crate::ui::scrolllist::draw_list_block;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
// use DatabaseTreelist::{DatabaseTree, DatabaseTreeItem};
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

/// `tree_files` returns a list of `DatabaseTree`
#[derive(Debug, PartialEq, Clone)]
pub struct TreeFile {
    /// path of this file
    pub path: std::path::PathBuf,
    /// unix filemode
    pub filemode: i32,
}

const FOLDER_ICON_COLLAPSED: &str = "\u{25b8}"; //▸
const FOLDER_ICON_EXPANDED: &str = "\u{25be}"; //▾
const EMPTY_STR: &str = "";

pub struct RevisionFilesComponent {
    pub tree: DatabaseTree,
    pub scroll: VerticalScroll,
}

impl RevisionFilesComponent {
    ///
    pub fn new() -> Self {
        Self {
            tree: DatabaseTree::default(),
            scroll: VerticalScroll::new(),
        }
    }

    fn tree_item_to_span<'a>(item: &'a DatabaseTreeItem, selected: bool, width: u16) -> Span<'a> {
        let path = item.info().path_str();
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

        let path = format!(
            "{}{}{:w$}",
            indent_str,
            path_arrow,
            path,
            w = width as usize
        );
        Span::styled(
            path,
            if selected {
                Style::default().fg(Color::Magenta).bg(Color::Green)
            } else {
                Style::default()
            },
        )
    }

    fn draw_tree<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
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

        let title = "Tree";
        draw_list_block(
            f,
            area,
            Block::default()
                .title(Span::styled(title, Style::default()))
                .borders(Borders::ALL)
                .border_style(Style::default()),
            items,
        );
        // draw_list(f, area, "hoge", items, true);

        self.scroll.draw(f, area);
    }
}

impl DrawableComponent for RevisionFilesComponent {
    fn draw<B: Backend>(&self, f: &mut Frame<B>, area: Rect) -> Result<()> {
        if true {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(area);

            self.draw_tree(f, chunks[0]);
        }
        Ok(())
    }
}

impl Component for RevisionFilesComponent {
    fn commands(&self, _out: &mut Vec<CommandInfo>, _force_all: bool) -> CommandBlocking {
        CommandBlocking::PassingOn
    }

    fn event(&mut self, event: Ev<Key>) -> Result<EventState> {
        if let crate::event::Event::Input(key) = event {
            if tree_nav(&mut self.tree, key) {
                return Ok(EventState::Consumed);
            } else if key == Key::Enter {
                println!("hgoehgoehgoeh")
            }
        }

        Ok(EventState::NotConsumed)
    }
}

//TODO: reuse for other tree usages
fn tree_nav(tree: &mut DatabaseTree, key: Key) -> bool {
    let tree_collapse_recursive = KeyEvent {
        code: KeyCode::Left,
        modifiers: KeyModifiers::SHIFT,
    };
    let tree_expand_recursive = KeyEvent {
        code: KeyCode::Right,
        modifiers: KeyModifiers::SHIFT,
    };
    if let Some(common_nav) = common_nav(key) {
        tree.move_selection(common_nav)
    } else if key == Key::from(tree_collapse_recursive) {
        tree.collapse_recursive();
        true
    } else if key == Key::from(tree_expand_recursive) {
        tree.expand_recursive();
        true
    } else {
        false
    }
}
