use crate::app::{App, Focus};
use crate::components::Component as _;
use crate::event::Key;

pub async fn handler(key: Key, app: &mut App) -> anyhow::Result<()> {
    match key {
        Key::Left => app.focus = Focus::DabataseList,
        Key::Char('c') => app.focus = Focus::ConnectionList,
        Key::Char('y') => {
            if let Some(text) = app.structure_table.selected_cell() {
                app.clipboard.store(text)
            }
        }
        key => app.structure_table.event(key)?,
    }
    Ok(())
}
