use crate::app::{App, FocusBlock};
use crate::event::Key;

pub async fn handler(key: Key, app: &mut App) -> anyhow::Result<()> {
    match key {
        Key::Char('h') => app.record_table.previous_column(),
        Key::Char('j') => app.record_table.next(1),
        Key::Ctrl('d') => app.record_table.next(10),
        Key::Char('k') => app.record_table.previous(1),
        Key::Ctrl('u') => app.record_table.previous(10),
        Key::Char('g') => app.record_table.scroll_top(),
        Key::Shift('G') | Key::Shift('g') => app.record_table.scroll_bottom(),
        Key::Char('l') => app.record_table.next_column(),
        Key::Left => app.focus_block = FocusBlock::DabataseList,
        Key::Char('c') => app.focus_block = FocusBlock::ConnectionList,
        _ => (),
    }
    Ok(())
}
