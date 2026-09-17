use std::time::Duration;

use serde_json::{Map, Value};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteRow},
    Column, Row, SqlitePool, TypeInfo, ValueRef,
};
use tauri::{AppHandle, Manager, Runtime};

/// Open a feature-owned SQLite database in the application configuration directory.
pub async fn connect_feature_database<R: Runtime>(
    app: &AppHandle<R>,
    filename: &str,
) -> Result<SqlitePool, String> {
    let directory = app
        .path()
        .app_config_dir()
        .map_err(|error| error.to_string())?;
    std::fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let options = SqliteConnectOptions::new()
        .filename(directory.join(filename))
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_secs(5));
    SqlitePoolOptions::new()
        .max_connections(4)
        .connect_with(options)
        .await
        .map_err(|error| error.to_string())
}

/// Convert fixed SQLx query rows into JSON-compatible values for typed feature
/// query commands. SQL text never crosses IPC; each feature owns its statements.
pub fn rows_to_json(rows: Vec<SqliteRow>) -> Result<Vec<Value>, String> {
    rows.iter().map(row_to_json).collect()
}

pub fn row_to_json(row: &SqliteRow) -> Result<Value, String> {
    let mut object = Map::with_capacity(row.len());
    for (index, column) in row.columns().iter().enumerate() {
        let raw = row.try_get_raw(index).map_err(|error| error.to_string())?;
        let value = if raw.is_null() {
            Value::Null
        } else {
            match raw.type_info().name() {
                "INTEGER" => Value::from(
                    row.try_get::<i64, _>(index)
                        .map_err(|error| error.to_string())?,
                ),
                "REAL" => Value::from(
                    row.try_get::<f64, _>(index)
                        .map_err(|error| error.to_string())?,
                ),
                "BLOB" => Value::from(
                    row.try_get::<Vec<u8>, _>(index)
                        .map_err(|error| error.to_string())?,
                ),
                _ => Value::from(
                    row.try_get::<String, _>(index)
                        .map_err(|error| error.to_string())?,
                ),
            }
        };
        object.insert(column.name().to_owned(), value);
    }
    Ok(Value::Object(object))
}
