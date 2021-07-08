use crate::app::{App, FocusBlock};
use crate::components::Component as _;
use crate::event::Key;
use crate::utils::convert_column_value_to_string;
use futures::TryStreamExt;
use regex::Regex;
use sqlx::Row;

pub async fn handler(key: Key, app: &mut App) -> anyhow::Result<()> {
    match key {
        Key::Enter => {
            let re = Regex::new(r"select .+ from ([^ ]+).*").unwrap();
            match re.captures(app.query.input.as_str()) {
                Some(caps) => {
                    let mut rows =
                        sqlx::query(app.query.input.as_str()).fetch(app.pool.as_ref().unwrap());
                    let headers =
                        sqlx::query(format!("desc `{}`", caps.get(1).unwrap().as_str()).as_str())
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
                    app.record_table.reset(headers, records);
                }
                None => {
                    sqlx::query(app.query.input.as_str())
                        .execute(app.pool.as_ref().unwrap())
                        .await?;
                }
            }
        }
        Key::Esc => app.focus_block = FocusBlock::DabataseList,
        key => app.query.event(key)?,
    }
    Ok(())
}
