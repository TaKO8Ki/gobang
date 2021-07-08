use crate::app::{App, FocusBlock};
use crate::components::Component as _;
use crate::event::Key;

pub async fn handler(key: Key, app: &mut App) -> anyhow::Result<()> {
    match key {
        Key::Left => app.focus_block = FocusBlock::DabataseList,
        Key::Char('c') => app.focus_block = FocusBlock::ConnectionList,
        key => app.structure_table.event(key)?,
    }
    Ok(())
}
