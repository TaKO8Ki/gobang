pub mod command;
pub mod revision_files;
pub mod utils;

pub use command::{CommandInfo, CommandText};
pub use revision_files::RevisionFilesComponent;

use anyhow::Result;
use crossterm::event::Event;
use std::convert::From;
use tui::{
    backend::Backend,
    layout::{Alignment, Rect},
    style::Style,
    text::{Span, Text},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame,
};

/// creates accessors for a list of components
///
/// allows generating code to make sure
/// we always enumerate all components in both getter functions
#[macro_export]
macro_rules! accessors {
    ($self:ident, [$($element:ident),+]) => {
        fn components(& $self) -> Vec<&dyn Component> {
            vec![
                $(&$self.$element,)+
            ]
        }

        fn components_mut(&mut $self) -> Vec<&mut dyn Component> {
            vec![
                $(&mut $self.$element,)+
            ]
        }
    };
}

/// creates a function to determine if any popup is visible
#[macro_export]
macro_rules! any_popup_visible {
    ($self:ident, [$($element:ident),+]) => {
        fn any_popup_visible(& $self) -> bool{
            ($($self.$element.is_visible()) || +)
        }
    };
}

/// creates the draw popup function
#[macro_export]
macro_rules! draw_popups {
    ($self:ident, [$($element:ident),+]) => {
        fn draw_popups<B: Backend>(& $self, mut f: &mut Frame<B>) -> Result<()>{
            //TODO: move the layout part out and feed it into `draw_popups`
            let size = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Min(1),
                    Constraint::Length($self.cmdbar.borrow().height()),
                ]
                .as_ref(),
            )
            .split(f.size())[0];

            ($($self.$element.draw(&mut f, size)?) , +);

            return Ok(());
        }
    };
}

/// simply calls
/// any_popup_visible!() and draw_popups!() macros
#[macro_export]
macro_rules! setup_popups {
    ($self:ident, [$($element:ident),+]) => {
        crate::any_popup_visible!($self, [$($element),+]);
        crate::draw_popups!($self, [ $($element),+ ]);
    };
}

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

///
#[derive(PartialEq)]
pub enum CommandBlocking {
    Blocking,
    PassingOn,
}

///
pub trait DrawableComponent {
    ///
    fn draw<B: Backend>(&self, f: &mut Frame<B>, rect: Rect) -> Result<()>;
}

///
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
    ///
    fn commands(&self, out: &mut Vec<CommandInfo>, force_all: bool) -> CommandBlocking;

    ///
    fn event(&mut self, ev: crate::event::Event<crate::event::Key>) -> Result<EventState>;

    ///
    fn focused(&self) -> bool {
        false
    }
    /// focus/unfocus this component depending on param
    fn focus(&mut self, _focus: bool) {}
    ///
    fn is_visible(&self) -> bool {
        true
    }
    ///
    fn hide(&mut self) {}
    ///
    fn show(&mut self) -> Result<()> {
        Ok(())
    }

    ///
    fn toggle_visible(&mut self) -> Result<()> {
        if self.is_visible() {
            self.hide();
            Ok(())
        } else {
            self.show()
        }
    }
}
