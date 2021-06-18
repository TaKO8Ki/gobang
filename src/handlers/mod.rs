pub mod database_list;
pub mod record_table;
pub mod table_list;

use crate::app::App;
use crossterm::event::KeyCode;

pub fn handle_app(key: KeyCode, app: &mut App) {
    match key {
        KeyCode::Char('e') => (),
        _ => (),
    }
}
