pub mod command;
pub mod completion;
pub mod connections;
pub mod databases;
pub mod error;
pub mod help;
pub mod record_table;
pub mod tab;
pub mod table;
pub mod table_filter;
pub mod table_status;
pub mod table_value;
pub mod utils;

pub use command::{CommandInfo, CommandText};
pub use completion::CompletionComponent;
pub use connections::ConnectionsComponent;
pub use databases::DatabasesComponent;
pub use error::ErrorComponent;
pub use help::HelpComponent;
pub use record_table::RecordTableComponent;
pub use tab::TabComponent;
pub use table::TableComponent;
pub use table_filter::TableFilterComponent;
pub use table_status::TableStatusComponent;
pub use table_value::TableValueComponent;

use anyhow::Result;
use async_trait::async_trait;
use std::convert::TryInto;
use tui::{backend::Backend, layout::Rect, Frame};
use unicode_width::UnicodeWidthChar;

#[derive(PartialEq)]
pub enum EventState {
    Consumed,
    NotConsumed,
}

impl EventState {
    pub fn is_consumed(&self) -> bool {
        *self == Self::Consumed
    }
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

pub trait DrawableComponent {
    fn draw<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect, focused: bool) -> Result<()>;
}

pub trait MovableComponent {
    fn draw<B: Backend>(
        &mut self,
        f: &mut Frame<B>,
        rect: Rect,
        focused: bool,
        x: u16,
        y: u16,
    ) -> Result<()>;
}

/// base component trait
#[async_trait]
pub trait Component {
    fn commands(&self, out: &mut Vec<CommandInfo>);

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

fn compute_character_width(c: char) -> u16 {
    UnicodeWidthChar::width(c).unwrap().try_into().unwrap()
}
