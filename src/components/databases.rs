use super::{
    compute_character_width, utils::scroll_vertical::VerticalScroll, Component,
    DatabaseFilterComponent, DrawableComponent, EventState,
};
use crate::components::command::{self, CommandInfo};
use crate::config::KeyConfig;
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
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use unicode_width::UnicodeWidthStr;

// ▸
const FOLDER_ICON_COLLAPSED: &str = "\u{25b8}";
// ▾
const FOLDER_ICON_EXPANDED: &str = "\u{25be}";
const EMPTY_STR: &str = "";

#[derive(PartialEq)]
pub enum Focus {
    Filter,
    Tree,
}

pub struct DatabasesComponent {
    tree: DatabaseTree,
    filter: DatabaseFilterComponent,
    filterd_tree: Option<DatabaseTree>,
    scroll: VerticalScroll,
    input: Vec<char>,
    input_idx: usize,
    input_cursor_position: u16,
    focus: Focus,
    key_config: KeyConfig,
}

impl DatabasesComponent {
    pub fn new(key_config: KeyConfig) -> Self {
        Self {
            tree: DatabaseTree::default(),
            filter: DatabaseFilterComponent::new(),
            filterd_tree: None,
            scroll: VerticalScroll::new(false, false),
            input: Vec::new(),
            input_idx: 0,
            input_cursor_position: 0,
            focus: Focus::Tree,
            key_config,
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
        self.input_cursor_position = 0;
        Ok(())
    }

    pub fn tree_focused(&self) -> bool {
        matches!(self.focus, Focus::Tree)
    }

    pub fn tree(&self) -> &DatabaseTree {
        self.filterd_tree.as_ref().unwrap_or(&self.tree)
    }

    fn tree_item_to_span(
        item: DatabaseTreeItem,
        selected: bool,
        width: u16,
        filter: Option<String>,
    ) -> Spans<'static> {
        let name = item.kind().name();
        let indent = item.info().indent();

        let indent_str = if indent == 0 {
            String::from("")
        } else {
            format!("{:w$}", " ", w = (indent as usize) * 2)
        };

        let arrow = if item.kind().is_database() || item.kind().is_schema() {
            if item.kind().is_database_collapsed() || item.kind().is_schema_collapsed() {
                FOLDER_ICON_COLLAPSED
            } else {
                FOLDER_ICON_EXPANDED
            }
        } else {
            EMPTY_STR
        };

        if let Some(filter) = filter {
            if item.kind().is_table() && name.contains(&filter) {
                let (first, rest) = &name.split_at(name.find(filter.as_str()).unwrap_or(0));
                let (middle, last) = &rest.split_at(filter.len().clamp(0, rest.len()));
                return Spans::from(vec![
                    Span::styled(
                        format!("{}{}{}", indent_str, arrow, first),
                        if selected {
                            Style::default().bg(Color::Blue)
                        } else {
                            Style::default()
                        },
                    ),
                    Span::styled(
                        middle.to_string(),
                        if selected {
                            Style::default().bg(Color::Blue).fg(Color::Blue)
                        } else {
                            Style::default().fg(Color::Blue)
                        },
                    ),
                    Span::styled(
                        format!("{:w$}", last.to_string(), w = width as usize),
                        if selected {
                            Style::default().bg(Color::Blue)
                        } else {
                            Style::default()
                        },
                    ),
                ]);
            }
        }

        Spans::from(Span::styled(
            format!("{}{}{:w$}", indent_str, arrow, name, w = width as usize),
            if selected {
                Style::default().bg(Color::Blue)
            } else {
                Style::default()
            },
        ))
    }

    fn draw_tree<B: Backend>(&self, f: &mut Frame<B>, area: Rect, focused: bool) -> Result<()> {
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

        // let filter = Paragraph::new(Span::styled(
        //     format!(
        //         "{}{:w$}",
        //         if self.input.is_empty() && matches!(self.focus, Focus::Tree) {
        //             "Filter tables".to_string()
        //         } else {
        //             self.input_str()
        //         },
        //         w = area.width as usize
        //     ),
        //     if let Focus::Filter = self.focus {
        //         Style::default()
        //     } else {
        //         Style::default().fg(Color::DarkGray)
        //     },
        // ))
        // .block(Block::default().borders(Borders::BOTTOM));
        // f.render_widget(filter, chunks[0]);
        // self.filter
        //     .draw(f, chunks[0], matches!(self.focus, Focus::Filter))?;

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
            .map(|(item, selected)| {
                Self::tree_item_to_span(
                    item.clone(),
                    selected,
                    area.width,
                    if self.input.is_empty() {
                        None
                    } else {
                        Some(self.input_str())
                    },
                )
            });

        draw_list_block(f, chunks[1], Block::default().borders(Borders::NONE), items);
        self.scroll.draw(f, chunks[1]);

        Ok(())
    }
}

impl DrawableComponent for DatabasesComponent {
    fn draw<B: Backend>(&self, f: &mut Frame<B>, area: Rect, focused: bool) -> Result<()> {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(100)].as_ref())
            .split(area);

        self.draw_tree(f, chunks[0], focused)?;
        Ok(())
    }
}

impl Component for DatabasesComponent {
    fn commands(&self, out: &mut Vec<CommandInfo>) {
        out.push(CommandInfo::new(command::expand_collapse(&self.key_config)))
    }

    fn event(&mut self, key: Key) -> Result<EventState> {
        let input_str: String = self.input.iter().collect();
        if key == self.key_config.filter && self.focus == Focus::Tree {
            self.focus = Focus::Filter;
            return Ok(EventState::Consumed);
        }

        match key {
            // Key::Char(c) if self.focus == Focus::Filter => {
            //     self.input.insert(self.input_idx, c);
            //     self.input_idx += 1;
            //     self.input_cursor_position += compute_character_width(c);
            //     self.filterd_tree = Some(self.tree.filter(self.input_str()));
            //     return Ok(EventState::Consumed);
            // }
            // Key::Delete | Key::Backspace if matches!(self.focus, Focus::Filter) => {
            //     if input_str.width() > 0 {
            //         if !self.input.is_empty() && self.input_idx > 0 {
            //             let last_c = self.input.remove(self.input_idx - 1);
            //             self.input_idx -= 1;
            //             self.input_cursor_position -= compute_character_width(last_c);
            //         }

            //         self.filterd_tree = if self.input.is_empty() {
            //             None
            //         } else {
            //             Some(self.tree.filter(self.input_str()))
            //         };
            //         return Ok(EventState::Consumed);
            //     }
            // }
            // Key::Left if matches!(self.focus, Focus::Filter) => {
            //     if !self.input.is_empty() && self.input_idx > 0 {
            //         self.input_idx -= 1;
            //         self.input_cursor_position = self
            //             .input_cursor_position
            //             .saturating_sub(compute_character_width(self.input[self.input_idx]));
            //     }
            //     return Ok(EventState::Consumed);
            // }
            // Key::Ctrl('a') => {
            //     if !self.input.is_empty() && self.input_idx > 0 {
            //         self.input_idx = 0;
            //         self.input_cursor_position = 0
            //     }
            //     return Ok(EventState::Consumed);
            // }
            // Key::Right if matches!(self.focus, Focus::Filter) => {
            //     if self.input_idx < self.input.len() {
            //         let next_c = self.input[self.input_idx];
            //         self.input_idx += 1;
            //         self.input_cursor_position += compute_character_width(next_c);
            //     }
            //     return Ok(EventState::Consumed);
            // }
            // Key::Ctrl('e') => {
            //     if self.input_idx < self.input.len() {
            //         self.input_idx = self.input.len();
            //         self.input_cursor_position = self.input_str().width() as u16;
            //     }
            //     return Ok(EventState::Consumed);
            // }
            Key::Enter if matches!(self.focus, Focus::Filter) => {
                self.focus = Focus::Tree;
                return Ok(EventState::Consumed);
            }
            key if matches!(self.focus, Focus::Filter) => {
                if self.filter.event(key)?.is_consumed() {
                    return Ok(EventState::Consumed);
                }
            }
            key => {
                if tree_nav(
                    if let Some(tree) = self.filterd_tree.as_mut() {
                        tree
                    } else {
                        &mut self.tree
                    },
                    key,
                    &self.key_config,
                ) {
                    return Ok(EventState::Consumed);
                }
            }
        }
        Ok(EventState::NotConsumed)
    }
}

fn tree_nav(tree: &mut DatabaseTree, key: Key, key_config: &KeyConfig) -> bool {
    if let Some(common_nav) = common_nav(key, key_config) {
        tree.move_selection(common_nav)
    } else {
        false
    }
}

#[cfg(test)]
mod test {
    use super::{Color, Database, DatabaseTreeItem, DatabasesComponent, Span, Spans, Style};
    use database_tree::Table;

    #[test]
    fn test_tree_database_tree_item_to_span() {
        const WIDTH: u16 = 10;
        assert_eq!(
            DatabasesComponent::tree_item_to_span(
                DatabaseTreeItem::new_database(
                    &Database {
                        name: "foo".to_string(),
                        children: Vec::new(),
                    },
                    false,
                ),
                false,
                WIDTH,
                None,
            ),
            Spans::from(vec![Span::raw(format!(
                "\u{25b8}{:w$}",
                "foo",
                w = WIDTH as usize
            ))])
        );

        assert_eq!(
            DatabasesComponent::tree_item_to_span(
                DatabaseTreeItem::new_database(
                    &Database {
                        name: "foo".to_string(),
                        children: Vec::new(),
                    },
                    false,
                ),
                true,
                WIDTH,
                None,
            ),
            Spans::from(vec![Span::styled(
                format!("\u{25b8}{:w$}", "foo", w = WIDTH as usize),
                Style::default().bg(Color::Blue)
            )])
        );
    }

    #[test]
    fn test_tree_table_tree_item_to_span() {
        const WIDTH: u16 = 10;
        assert_eq!(
            DatabasesComponent::tree_item_to_span(
                DatabaseTreeItem::new_table(
                    &Database {
                        name: "foo".to_string(),
                        children: Vec::new(),
                    },
                    &Table {
                        name: "bar".to_string(),
                        create_time: None,
                        update_time: None,
                        engine: None,
                        schema: None
                    },
                ),
                false,
                WIDTH,
                None,
            ),
            Spans::from(vec![Span::raw(format!(
                "  {:w$}",
                "bar",
                w = WIDTH as usize
            ))])
        );

        assert_eq!(
            DatabasesComponent::tree_item_to_span(
                DatabaseTreeItem::new_table(
                    &Database {
                        name: "foo".to_string(),
                        children: Vec::new(),
                    },
                    &Table {
                        name: "bar".to_string(),
                        create_time: None,
                        update_time: None,
                        engine: None,
                        schema: None
                    },
                ),
                true,
                WIDTH,
                None,
            ),
            Spans::from(Span::styled(
                format!("  {:w$}", "bar", w = WIDTH as usize),
                Style::default().bg(Color::Blue),
            ))
        );
    }

    #[test]
    fn test_filterd_tree_item_to_span() {
        const WIDTH: u16 = 10;
        assert_eq!(
            DatabasesComponent::tree_item_to_span(
                DatabaseTreeItem::new_table(
                    &Database {
                        name: "foo".to_string(),
                        children: Vec::new(),
                    },
                    &Table {
                        name: "barbaz".to_string(),
                        create_time: None,
                        update_time: None,
                        engine: None,
                        schema: None
                    },
                ),
                false,
                WIDTH,
                Some("rb".to_string()),
            ),
            Spans::from(vec![
                Span::raw(format!("  {}", "ba")),
                Span::styled("rb", Style::default().fg(Color::Blue)),
                Span::raw(format!("{:w$}", "az", w = WIDTH as usize))
            ])
        );

        assert_eq!(
            DatabasesComponent::tree_item_to_span(
                DatabaseTreeItem::new_table(
                    &Database {
                        name: "foo".to_string(),
                        children: Vec::new(),
                    },
                    &Table {
                        name: "barbaz".to_string(),
                        create_time: None,
                        update_time: None,
                        engine: None,
                        schema: None
                    },
                ),
                true,
                WIDTH,
                Some("rb".to_string()),
            ),
            Spans::from(vec![
                Span::styled(format!("  {}", "ba"), Style::default().bg(Color::Blue)),
                Span::styled("rb", Style::default().bg(Color::Blue).fg(Color::Blue)),
                Span::styled(
                    format!("{:w$}", "az", w = WIDTH as usize),
                    Style::default().bg(Color::Blue)
                )
            ])
        );
    }
}
