use crate::{components::ScrollType, ui::scrollbar::draw_scrollbar};
use std::{cell::Cell, thread::panicking};
use tui::{backend::Backend, layout::Rect, Frame};

pub struct HorizontalScroll {
    right: Cell<usize>,
    max_right: Cell<usize>,
}

impl HorizontalScroll {
    pub const fn new() -> Self {
        Self {
            right: Cell::new(0),
            max_right: Cell::new(0),
        }
    }

    pub fn get_right(&self) -> usize {
        self.right.get()
    }

    pub fn reset(&self) {
        self.right.set(0);
    }

    pub fn _move_right(&self, move_type: ScrollType) -> bool {
        let old = self.right.get();
        let max = self.max_right.get();

        let new_scroll_right = match move_type {
            ScrollType::Down => old.saturating_add(1),
            ScrollType::Up => old.saturating_sub(1),
            ScrollType::Home => 0,
            ScrollType::End => max,
            _ => old,
        };

        let new_scroll_right = new_scroll_right.clamp(0, max);

        if new_scroll_right == old {
            return false;
        }

        self.right.set(new_scroll_right);

        true
    }

    pub fn update(&self, selection: usize, selection_max: usize, visual_width: usize) -> usize {
        let new_right = calc_scroll_right(self.get_right(), visual_width, selection, selection_max);
        self.right.set(new_right);

        if visual_width == 0 {
            self.max_right.set(0);
        } else {
            let new_max = selection_max.saturating_sub(visual_width);
            self.max_right.set(new_max);
        }

        new_right
    }

    pub fn _update_no_selection(&self, line_count: usize, visual_width: usize) -> usize {
        self.update(self.get_right(), line_count, visual_width)
    }

    pub fn draw<B: Backend>(&self, f: &mut Frame<B>, r: Rect) {
        draw_scrollbar(f, r, self.max_right.get(), self.right.get(), false);
    }
}

fn calc_scroll_right(
    current_right: usize,
    width_in_lines: usize,
    selection: usize,
    selection_max: usize,
) -> usize {
    if width_in_lines == 0 {
        return 0;
    }
    if selection_max <= width_in_lines {
        return 0;
    }

    if current_right + width_in_lines <= selection {
        selection.saturating_sub(width_in_lines) + 1
    } else if current_right > selection {
        selection
    } else {
        current_right
    }
}
