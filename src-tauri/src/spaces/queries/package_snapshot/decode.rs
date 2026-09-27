use serde_json::Value;
use sqlx::{sqlite::SqliteRow, Row};

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
    serde_json::from_str(&decode_text(row, key)?).map_err(db::json_error)
}

pub(super) fn decode_optional_json(row: &SqliteRow, key: &str) -> Result<Option<Value>, String> {
    decode_optional_text(row, key)?
        .map(|value| serde_json::from_str(&value).map_err(db::json_error))
        .transpose()
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn nullable_json_stays_absent_and_malformed_json_is_rejected() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("database");
        let row = sqlx::query(
            "SELECT NULL AS absent, '{broken' AS damaged, '{\"title\":\"Research\"}' AS content",
        )
        .fetch_one(&pool)
        .await
        .expect("row");
        assert_eq!(
            super::decode_optional_json(&row, "absent").expect("optional"),
            None
        );
        assert_eq!(
            super::decode_json(&row, "content").expect("content")["title"],
            "Research"
        );
        assert!(super::decode_json(&row, "damaged").is_err());
        assert!(super::decode_optional_json(&row, "damaged").is_err());
    }
}
