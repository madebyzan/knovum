use serde_json::Value;
use sqlx::{Row, SqlitePool};

use crate::spaces::{actions::ActionInvocationOutcome, db, runs::AttemptClaim};

pub(super) async fn reusable(
    pool: &SqlitePool,
    row: &sqlx::sqlite::SqliteRow,
) -> Result<Option<ActionInvocationOutcome>, String> {
    let state: String = row.try_get("state").map_err(db::sql_error)?;
    if state == "succeeded" {
        let encoded: String = row.try_get("output").map_err(db::sql_error)?;
        return Ok(Some(ActionInvocationOutcome::QueryResult {
            result: serde_json::from_str(&encoded).map_err(db::json_error)?,
        }));
    }
    let child_id: Option<String> = row.try_get("child_run_id").map_err(db::sql_error)?;
    let Some(child_id) = child_id else {
        return Ok(None);
    };
    let child_state: Option<String> =
        sqlx::query_scalar("SELECT state FROM spaces_runs WHERE id=$1")
            .bind(&child_id)
            .fetch_optional(pool)
            .await
            .map_err(db::sql_error)?;
    Ok(match child_state.as_deref() {
        Some("queued" | "running" | "waiting" | "paused") => {
            Some(ActionInvocationOutcome::AcceptedRun { run_id: child_id })
        }
        Some("succeeded" | "completed_with_errors") => {
            let output: Option<String> = sqlx::query_scalar(
                "SELECT output FROM spaces_run_steps WHERE run_id=$1 AND state='succeeded'
                 ORDER BY updated_at DESC LIMIT 1",
            )
            .bind(&child_id)
            .fetch_optional(pool)
            .await
            .map_err(db::sql_error)?
            .flatten();
            Some(ActionInvocationOutcome::QueryResult {
                result: output
                    .map(|encoded| serde_json::from_str(&encoded))
                    .transpose()
                    .map_err(db::json_error)?
                    .unwrap_or(Value::Null),
            })
        }
        _ => None,
    })
}

pub(super) fn lineage(claim: &AttemptClaim) -> &str {
    claim
        .policy_snapshot
        .get("retryRootRunId")
        .and_then(Value::as_str)
        .unwrap_or(&claim.run_id)
}

pub(super) fn node_id(claim: &AttemptClaim) -> &str {
    claim
        .configuration
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or(&claim.step_instance_id)
}
