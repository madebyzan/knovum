use sha2::Digest;
use sqlx::{types::Json, Row};

use crate::spaces::{db, runs::AttemptClaim};

pub(super) fn claim(
    row: &sqlx::sqlite::SqliteRow,
    attempt_id: String,
    step_instance_id: String,
    run_id: String,
) -> Result<AttemptClaim, String> {
    Ok(AttemptClaim {
        attempt_id,
        run_id,
        step_instance_id,
        execution_epoch: row.try_get("execution_epoch").map_err(db::sql_error)?,
        attempt_number: row.try_get("attempt_number").map_err(db::sql_error)?,
        scope: super::super::super::run_rows::parse_scope(row)?,
        node_kind: row.try_get("node_kind").map_err(db::sql_error)?,
        configuration: json(row, "configuration")?,
        resolved_input: json(row, "resolved_input")?,
        policy_snapshot: json(row, "policy_snapshot")?,
        budget: json(row, "budget")?,
        cancellation: tokio_util::sync::CancellationToken::new(),
    })
}

fn json(row: &sqlx::sqlite::SqliteRow, field: &str) -> Result<serde_json::Value, String> {
    row.try_get::<Json<serde_json::Value>, _>(field)
        .map(|Json(value)| value)
        .map_err(db::sql_error)
}

pub(super) fn request_digest(encoded: &str) -> String {
    hex::encode(sha2::Sha256::digest(encoded.as_bytes()))
}
