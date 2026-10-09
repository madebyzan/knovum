use serde_json::Value;
use sqlx::{sqlite::SqliteRow, types::Json, Row};

use crate::spaces::db;

pub(super) fn decode_text(row: &SqliteRow, key: &str) -> Result<String, String> {
    row.try_get(key).map_err(db::sql_error)
}

pub(super) fn decode_optional_text(row: &SqliteRow, key: &str) -> Result<Option<String>, String> {
    row.try_get(key).map_err(db::sql_error)
}

pub(super) fn decode_integer(row: &SqliteRow, key: &str) -> Result<i64, String> {
    row.try_get(key).map_err(db::sql_error)
}

pub(super) fn decode_optional_integer(row: &SqliteRow, key: &str) -> Result<Option<i64>, String> {
    row.try_get(key).map_err(db::sql_error)
}

pub(super) fn decode_json(row: &SqliteRow, key: &str) -> Result<Value, String> {
    row.try_get::<Json<Value>, _>(key)
        .map(|Json(value)| value)
        .map_err(db::sql_error)
}

pub(super) fn decode_optional_json(row: &SqliteRow, key: &str) -> Result<Option<Value>, String> {
    row.try_get::<Option<Json<Value>>, _>(key)
        .map(|value| value.map(|Json(value)| value))
        .map_err(db::sql_error)
}

#[cfg(test)]
mod tests;
