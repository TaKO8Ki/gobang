pub mod command;
pub mod databases;
pub mod query;
pub mod table;
pub mod utils;

pub use command::{CommandInfo, CommandText};
pub use databases::DatabasesComponent;
pub use table::TableComponent;

use anyhow::Result;
use tui::{backend::Backend, layout::Rect, Frame};

#[derive(Copy, Clone)]
pub enum ScrollType {
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
}

#[derive(Copy, Clone)]
pub enum Direction {
    Up,
    Down,
}

pub trait DrawableComponent {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect, focused: bool) -> Result<()>;
}

/// base component trait
pub trait Component {
    fn event(&mut self, key: crate::event::Key) -> Result<()>;

    fn focused(&self) -> bool {
        false
    }

    fn focus(&mut self, _focus: bool) {}

    fn is_visible(&self) -> bool {
        true
    }

    fn hide(&mut self) {}

    fn show(&mut self) -> Result<()> {
        Ok(())
    }

    fn toggle_visible(&mut self) -> Result<()> {
        if self.is_visible() {
            self.hide();
            Ok(())
        } else {
            self.show()
        }
    }
}
