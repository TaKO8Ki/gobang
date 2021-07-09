use crate::app::{App, FocusBlock};
use crate::components::Component as _;
use crate::event::Key;

pub async fn handler(key: Key, app: &mut App) -> anyhow::Result<()> {
    match key {
        Key::Left => app.focus_block = FocusBlock::DabataseList,
        Key::Char('c') => app.focus_block = FocusBlock::ConnectionList,
        Key::Char('y') => {
            if let Some(text) = app.record_table.selected_cell() {
                app.clipboard.store(text)
            }
        }
        key => app.record_table.event(key)?,
    }
    Ok(())
}
