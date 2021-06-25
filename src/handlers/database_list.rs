use crate::app::{App, FocusBlock};
use crate::event::Key;

pub async fn handler(key: Key, app: &mut App, focused: bool) -> anyhow::Result<()> {
    if focused {
        match key {
            Key::Char('j') => app.next_database(),
            Key::Char('k') => app.previous_database(),
            Key::Esc => app.focus_type = FocusBlock::DabataseList(false),
            _ => (),
        }
    } else {
        match key {
            Key::Char('j') => app.focus_type = FocusBlock::TableList(false),
            Key::Char('l') => app.focus_type = FocusBlock::RecordTable(false),
            Key::Char('c') => app.focus_type = FocusBlock::ConnectionList,
            Key::Enter => app.focus_type = FocusBlock::DabataseList(true),
            _ => (),
        }
    }
    Ok(())
}
