use std::time::Duration;

use serde_json::{Map, Value};
use sqlx::{
    sqlite::{
        SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteRow, SqliteSynchronous,
    },
    Column, Row, SqlitePool, TypeInfo, ValueRef,
};
use tauri::{AppHandle, Manager, Runtime};

/// Open a feature-owned SQLite database in the application configuration directory.
///
/// The pragmas here are the storage contract every feature database shares. `synchronous=NORMAL`
/// is the setting WAL is designed for: a commit no longer waits for an fsync, the file stays safe
/// from corruption, and the only exposure is that the last few transactions can be lost to a
/// power cut, which local recovery archives already cover. `FULL` made every edit pay a disk
/// flush. The cache and mmap windows keep a growing library's index pages in memory instead of
/// re-reading them, and temp tables stay out of the filesystem so the sorts that query plans ask
/// for do not touch disk.
pub async fn connect_feature_database<R: Runtime>(
    app: &AppHandle<R>,
    filename: &str,
) -> Result<SqlitePool, String> {
    crate::backup::ready(app).await?;
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
        .synchronous(SqliteSynchronous::Normal)
        .busy_timeout(Duration::from_secs(5))
        .pragma("cache_size", "-16384")
        .pragma("mmap_size", "268435456")
        .pragma("temp_store", "MEMORY");
    SqlitePoolOptions::new()
        .max_connections(4)
        .connect_with(options)
        .await
        .map_err(|error| error.to_string())
}

/// Refresh the planner's statistics for the indexes a feature actually queries.
///
/// Without statistics SQLite falls back to heuristics and can choose a Space-wide index where a
/// selective one exists. `analysis_limit` keeps the pass bounded, and `optimize` is a no-op when
/// nothing changed, so this is safe to run on every start.
pub async fn optimize(pool: &SqlitePool) {
    for statement in ["PRAGMA analysis_limit=400", "PRAGMA optimize"] {
        if let Err(error) = sqlx::query(statement).execute(pool).await {
            log::warn!("failed to refresh SQLite planner statistics: {error}");
        }
    }
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
