use crate::app::{App, FocusBlock};
use crate::event::Key;
use crate::utils::{get_columns, get_records};

pub async fn handler(key: Key, app: &mut App, focused: bool) -> anyhow::Result<()> {
    if focused {
        match key {
            Key::Char('j') => {
                if app.selected_database.selected().is_some() {
                    app.next_table();
                    app.record_table.column_index = 0;
                    if let Some(database) = app.selected_database() {
                        if let Some(table) = app.selected_table() {
                            let (headers, records) =
                                get_records(database, table, app.pool.as_ref().unwrap()).await?;
                            app.record_table.state.select(Some(0));
                            app.record_table.headers = headers;
                            app.record_table.rows = records;
                        }
                    }

                    app.structure_table.column_index = 0;
                    if let Some(database) = app.selected_database() {
                        if let Some(table) = app.selected_table() {
                            let (headers, records) =
                                get_columns(database, table, app.pool.as_ref().unwrap()).await?;
                            app.structure_table.state.select(Some(0));
                            app.structure_table.headers = headers;
                            app.structure_table.rows = records;
                        }
                    }
                }
            }
            Key::Char('k') => {
                if app.selected_database.selected().is_some() {
                    app.previous_table();
                    app.record_table.column_index = 0;
                    if let Some(database) = app.selected_database() {
                        if let Some(table) = app.selected_table() {
                            let (headers, records) =
                                get_records(database, table, app.pool.as_ref().unwrap()).await?;
                            app.record_table.state.select(Some(0));
                            app.record_table.headers = headers;
                            app.record_table.rows = records;
                        }
                    }

                    app.structure_table.column_index = 0;
                    if let Some(database) = app.selected_database() {
                        if let Some(table) = app.selected_table() {
                            let (headers, records) =
                                get_columns(database, table, app.pool.as_ref().unwrap()).await?;
                            app.structure_table.state.select(Some(0));
                            app.structure_table.headers = headers;
                            app.structure_table.rows = records;
                        }
                    }
                }
            }
            Key::Esc => app.focus_block = FocusBlock::TableList(false),
            _ => (),
        }
    } else {
        match key {
            Key::Char('k') => app.focus_block = FocusBlock::DabataseList(false),
            Key::Char('l') => app.focus_block = FocusBlock::RecordTable(false),
            Key::Char('c') => app.focus_block = FocusBlock::ConnectionList,
            Key::Enter => app.focus_block = FocusBlock::TableList(true),
            _ => (),
        }
    }
    Ok(())
}
