use std::iter::Iterator;

use tui::{
    backend::Backend,
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::Spans,
    widgets::{Block, List, ListItem, Widget},
    Frame,
};

struct ScrollableList<'b, L>
where
    L: Iterator<Item = Spans<'b>>,
{
    block: Option<Block<'b>>,
    items: L,
    style: Style,
}

impl<'b, L> ScrollableList<'b, L>
where
    L: Iterator<Item = Spans<'b>>,
{
    fn new(items: L) -> Self {
        Self {
            block: None,
            items,
            style: Style::default(),
        }
    }

    fn block(mut self, block: Block<'b>) -> Self {
        self.block = Some(block);
        self
    }
}

impl<'b, L> Widget for ScrollableList<'b, L>
where
    L: Iterator<Item = Spans<'b>>,
{
    fn render(self, area: Rect, buf: &mut Buffer) {
        List::new(self.items.map(ListItem::new).collect::<Vec<ListItem>>())
            .block(self.block.unwrap_or_default())
            .style(self.style)
            .render(area, buf);
    }
}

pub fn draw_list_block<'b, B: Backend, L>(f: &mut Frame<B>, r: Rect, block: Block<'b>, items: L)
where
    L: Iterator<Item = Spans<'b>>,
{
    let list = ScrollableList::new(items).block(block);
    f.render_widget(list, r);
}
