use crate::config::KeyConfig;
use crate::event::Key;
use database_tree::MoveSelection;

pub mod reflow;
pub mod scrollbar;
pub mod scrolllist;
pub mod stateful_paragraph;

pub fn common_nav(key: Key, key_config: &KeyConfig) -> Option<MoveSelection> {
    if key == key_config.scroll_down {
        Some(MoveSelection::Down)
    } else if key == key_config.scroll_up {
        Some(MoveSelection::Up)
    } else if key == key_config.scroll_down_multiple_lines {
        Some(MoveSelection::MultipleDown)
    } else if key == key_config.scroll_up_multiple_lines {
        Some(MoveSelection::MultipleUp)
    } else if key == key_config.scroll_right {
        Some(MoveSelection::Right)
    } else if key == key_config.scroll_left {
        Some(MoveSelection::Left)
    } else if key == key_config.scroll_to_top {
        Some(MoveSelection::Top)
    } else if key == key_config.scroll_to_bottom {
        Some(MoveSelection::End)
    } else if key == key_config.enter {
        Some(MoveSelection::Enter)
    } else {
        None
    }
}
