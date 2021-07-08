use crate::event::Key;
use database_tree::MoveSelection;

pub mod scrollbar;
pub mod scrolllist;

pub fn common_nav(key: Key) -> Option<MoveSelection> {
    if key == Key::Char('j') {
        Some(MoveSelection::Down)
    } else if key == Key::Char('k') {
        Some(MoveSelection::Up)
    } else if key == Key::PageUp {
        Some(MoveSelection::PageUp)
    } else if key == Key::PageDown {
        Some(MoveSelection::PageDown)
    } else if key == Key::Char('l') {
        Some(MoveSelection::Right)
    } else if key == Key::Char('h') {
        Some(MoveSelection::Left)
    } else {
        None
    }
}
