use sha2::Digest;
use sqlx::Row;

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
        home_space_id: row.try_get("home_space_id").map_err(db::sql_error)?,
        node_kind: row.try_get("node_kind").map_err(db::sql_error)?,
        configuration: json(row, "configuration")?,
        resolved_input: json(row, "resolved_input")?,
        policy_snapshot: json(row, "policy_snapshot")?,
        budget: json(row, "budget")?,
    })
}

fn json(row: &sqlx::sqlite::SqliteRow, field: &str) -> Result<serde_json::Value, String> {
    let encoded: String = row.try_get(field).map_err(db::sql_error)?;
    serde_json::from_str(&encoded).map_err(db::json_error)
}

pub(super) fn request_digest(encoded: &str) -> String {
    format!("{:x}", sha2::Sha256::digest(encoded.as_bytes()))
}
