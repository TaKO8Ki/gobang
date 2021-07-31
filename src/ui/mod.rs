use crate::config::KeyConfig;
use crate::event::Key;
use database_tree::MoveSelection;

pub mod scrollbar;
pub mod scrolllist;

pub fn common_nav(key: Key, key_config: &KeyConfig) -> Option<MoveSelection> {
    if key == key_config.scroll_down {
        Some(MoveSelection::Down)
    } else if key == key_config.scroll_up {
        Some(MoveSelection::Up)
    } else if key == Key::PageUp {
        Some(MoveSelection::PageUp)
    } else if key == Key::PageDown {
        Some(MoveSelection::PageDown)
    } else if key == key_config.scroll_right {
        Some(MoveSelection::Right)
    } else if key == key_config.scroll_left {
        Some(MoveSelection::Left)
    } else {
        None
    }
}
