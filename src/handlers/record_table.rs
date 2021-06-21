use crate::app::App;
use crate::event::Key;
use crate::utils::get_records;
use sqlx::mysql::MySqlPool;

pub async fn handler<'a>(key: Key, app: &mut App<'a>, pool: &MySqlPool) -> anyhow::Result<()> {
    let selected_database = app
        .databases
        .get(app.selected_database.selected().unwrap())
        .unwrap();
    let selected_table = selected_database
        .tables
        .get(selected_database.selected_table.selected().unwrap())
        .unwrap();
    let (headers, records) = get_records(selected_database, selected_table, pool).await?;
    app.record_table.headers = headers;
    app.record_table.rows = records;
    Ok(())
}
