use crate::app::{App, FocusBlock};
use crate::event::Key;

pub async fn handler(key: Key, app: &mut App) -> anyhow::Result<()> {
    match key {
        Key::Char('h') => app.record_table.previous_column(),
        Key::Char('j') => app.record_table.next(),
        Key::Char('k') => app.record_table.previous(),
        Key::Char('l') => app.record_table.next_column(),
        Key::Left => app.focus_block = FocusBlock::DabataseList,
        Key::Char('c') => app.focus_block = FocusBlock::ConnectionList,
        _ => (),
    }
    Ok(())
}
