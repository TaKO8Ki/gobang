pub mod command;
pub mod databases;
pub mod utils;

pub use command::{CommandInfo, CommandText};
pub use databases::DatabasesComponent;

use anyhow::Result;
use std::convert::From;
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

#[derive(PartialEq)]
pub enum CommandBlocking {
    Blocking,
    PassingOn,
}

pub trait DrawableComponent {
    ///
    fn draw<B: Backend>(&self, f: &mut Frame<B>, rect: Rect) -> Result<()>;
}

#[derive(PartialEq)]
pub enum EventState {
    Consumed,
    NotConsumed,
}

impl From<bool> for EventState {
    fn from(consumed: bool) -> Self {
        if consumed {
            Self::Consumed
        } else {
            Self::NotConsumed
        }
    }
}

/// base component trait
pub trait Component {
    fn commands(&self, out: &mut Vec<CommandInfo>, force_all: bool) -> CommandBlocking;

    fn event(&mut self, key: crate::event::Key) -> Result<EventState>;

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
