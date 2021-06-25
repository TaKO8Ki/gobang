use crate::app::{App, FocusBlock};
use crate::event::Key;

pub async fn handler(key: Key, app: &mut App, focused: bool) -> anyhow::Result<()> {
    if focused {
        match key {
            Key::Char('h') => app.record_table.previous_column(),
            Key::Char('j') => app.record_table.next(),
            Key::Char('k') => app.record_table.previous(),
            Key::Char('l') => app.record_table.next_column(),
            Key::Esc => app.focus_type = FocusBlock::RecordTable(false),
            _ => (),
        }
    } else {
        match key {
            Key::Char('h') => app.focus_type = FocusBlock::TableList(false),
            Key::Char('c') => app.focus_type = FocusBlock::ConnectionList,
            Key::Enter => app.focus_type = FocusBlock::RecordTable(true),
            _ => (),
        }
    }
    Ok(())
}
