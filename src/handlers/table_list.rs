use crate::app::{App, FocusBlock};
use crate::event::Key;
use crate::utils::get_records;

pub async fn handler(key: Key, app: &mut App, focused: bool) -> anyhow::Result<()> {
    if focused {
        match key {
            Key::Char('j') => match app.selected_database.selected() {
                Some(_) => {
                    app.record_table.column_index = 0;
                    app.next_table();
                    if let Some(database) = app.selected_database() {
                        if let Some(table) = app.selected_table() {
                            let (headers, records) =
                                get_records(database, table, app.pool.as_ref().unwrap()).await?;
                            app.record_table.state.select(Some(0));
                            app.record_table.headers = headers;
                            app.record_table.rows = records;
                        }
                    }
                }
                None => (),
            },
            Key::Char('k') => match app.selected_database.selected() {
                Some(_) => {
                    app.record_table.column_index = 0;
                    app.previous_table();
                    if let Some(database) = app.selected_database() {
                        if let Some(table) = app.selected_table() {
                            let (headers, records) =
                                get_records(database, table, app.pool.as_ref().unwrap()).await?;
                            app.record_table.state.select(Some(0));
                            app.record_table.headers = headers;
                            app.record_table.rows = records;
                        }
                    }
                }
                None => (),
            },
            Key::Esc => app.focus_type = FocusBlock::TableList(false),
            _ => (),
        }
    } else {
        match key {
            Key::Char('k') => app.focus_type = FocusBlock::DabataseList(false),
            Key::Char('l') => app.focus_type = FocusBlock::RecordTable(false),
            Key::Char('c') => app.focus_type = FocusBlock::ConnectionList,
            Key::Enter => app.focus_type = FocusBlock::TableList(true),
            _ => (),
        }
    }
    Ok(())
}
