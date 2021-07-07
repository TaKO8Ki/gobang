use crate::app::{App, FocusBlock};
use crate::event::Key;
use crate::utils::convert_column_value_to_string;
use futures::TryStreamExt;
use regex::Regex;
use sqlx::Row;
use unicode_width::UnicodeWidthStr;

pub async fn handler(key: Key, app: &mut App) -> anyhow::Result<()> {
    if true {
        match key {
            Key::Enter => {
                app.query = app.input.drain(..).collect();
                let re = Regex::new(r"select .+ from ([^ ]+).*").unwrap();
                match re.captures(app.query.as_str()) {
                    Some(caps) => {
                        let mut rows =
                            sqlx::query(app.query.as_str()).fetch(app.pool.as_ref().unwrap());
                        let headers = sqlx::query(
                            format!("desc `{}`", caps.get(1).unwrap().as_str()).as_str(),
                        )
                        .fetch_all(app.pool.as_ref().unwrap())
                        .await?
                        .iter()
                        .map(|table| table.get(0))
                        .collect::<Vec<String>>();
                        let mut records = vec![];
                        while let Some(row) = rows.try_next().await? {
                            records.push(
                                row.columns()
                                    .iter()
                                    .map(|col| convert_column_value_to_string(&row, col))
                                    .collect::<Vec<String>>(),
                            )
                        }
                        app.record_table.headers = headers;
                        app.record_table.rows = records;
                    }
                    None => {
                        sqlx::query(app.query.as_str())
                            .execute(app.pool.as_ref().unwrap())
                            .await?;
                    }
                }
            }
            Key::Char(c) => app.input.push(c),
            Key::Delete | Key::Backspace => {
                if app.input.width() > 0 {
                    if app.input_cursor_x == 0 {
                        app.input.pop();
                        return Ok(());
                    }
                    if app.input.width() - app.input_cursor_x as usize > 0 {
                        app.input
                            .remove(app.input.width() - app.input_cursor_x as usize);
                    }
                }
            }
            Key::Left => app.decrement_input_cursor_x(),
            Key::Right => app.increment_input_cursor_x(),
            Key::Esc => app.focus_block = FocusBlock::Query,
            _ => {}
        }
    } else {
        match key {
            Key::Char('h') => app.focus_block = FocusBlock::DabataseList,
            Key::Char('j') => app.focus_block = FocusBlock::RecordTable,
            Key::Char('c') => app.focus_block = FocusBlock::ConnectionList,
            Key::Enter => app.focus_block = FocusBlock::Query,
            _ => (),
        }
    }
    Ok(())
}
