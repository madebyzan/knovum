use serde_json::Value;
use sqlx::{Row, SqlitePool};

use super::scheduler;
use crate::spaces::db;

mod reconciliation;

pub(super) use reconciliation::reconcile;

#[derive(Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReconciliationDecision {
    Retry,
    Fail,
}

pub(super) async fn submit_input(
    pool: &SqlitePool,
    step_id: &str,
    expected_version: i64,
    value: &Value,
) -> Result<(), String> {
    validate_user_input(pool, step_id, value).await?;
    settle(pool, step_id, expected_version, "user_input", Ok(value)).await
}

async fn validate_user_input(
    pool: &SqlitePool,
    step_id: &str,
    value: &Value,
) -> Result<(), String> {
    let encoded = sqlx::query_scalar::<_, Option<String>>(
        "SELECT output FROM spaces_run_steps WHERE id=$1 AND state='waiting'
         AND wait_reason='user_input'",
    )
    .bind(step_id)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?
    .flatten()
    .ok_or("run-step-stale: user input checkpoint is unavailable")?;
    let checkpoint: Value = serde_json::from_str(&encoded).map_err(db::json_error)?;
    let schema = checkpoint
        .get("schema")
        .ok_or("user-input-schema-invalid: schema is missing")?;
    let validator = jsonschema::validator_for(schema)
        .map_err(|error| format!("user-input-schema-invalid: {error}"))?;
    if let Err(error) = validator.validate(value) {
        return Err(format!("user-input-invalid: {error}"));
    }
    Ok(())
}

pub(super) async fn decide_review(
    pool: &SqlitePool,
    step_id: &str,
    expected_version: i64,
    approved: bool,
) -> Result<(), String> {
    if approved {
        validate_review_revision(pool, step_id).await?;
    }
    let approved_value = Value::Bool(true);
    let outcome = if approved {
        Ok(&approved_value)
    } else {
        Err("content-review-rejected")
    };
    settle(pool, step_id, expected_version, "content_review", outcome).await
}

pub(super) async fn continue_debug_step(
    pool: &SqlitePool,
    step_id: &str,
    expected_version: i64,
) -> Result<(), String> {
    let now = crate::clock::unix_time_millis()?;
    let mut tx = db::begin_write(pool).await?;
    let run_id = sqlx::query_scalar::<_, String>(
        "SELECT run_id FROM spaces_run_steps WHERE id=$1 AND state='paused'
         AND wait_reason='debug_breakpoint' AND state_version=$2",
    )
    .bind(step_id)
    .bind(expected_version)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db::sql_error)?
    .ok_or("run-step-stale: debug breakpoint changed before continuation")?;
    sqlx::query(
        "UPDATE spaces_run_steps SET state='ready',wait_reason=NULL,state_version=state_version+1,
         updated_at=$1 WHERE id=$2",
    )
    .bind(now)
    .bind(step_id)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    sqlx::query(
        "UPDATE spaces_runs SET state='queued',wait_reason=NULL,state_version=state_version+1,
         updated_at=$1 WHERE id=$2 AND state='paused' AND completed_at IS NULL",
    )
    .bind(now)
    .bind(&run_id)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    super::append_event(
        &mut tx,
        &run_id,
        Some(step_id),
        None,
        crate::spaces::runs::RunEventKind::DebugBreakpointContinued,
        &serde_json::json!({}),
        now,
    )
    .await?;
    tx.commit().await.map_err(db::sql_error)
}

async fn validate_review_revision(pool: &SqlitePool, step_id: &str) -> Result<(), String> {
    let encoded: String = sqlx::query_scalar::<_, Option<String>>(
        "SELECT output FROM spaces_run_steps WHERE id=$1 AND state='waiting'
         AND wait_reason='content_review'",
    )
    .bind(step_id)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?
    .flatten()
    .ok_or("review-stale: review checkpoint is unavailable")?;
    let checkpoint: Value = serde_json::from_str(&encoded).map_err(db::json_error)?;
    let node_id = checkpoint
        .get("nodeId")
        .and_then(Value::as_str)
        .ok_or("review-stale: reviewed resource is missing")?;
    let expected = checkpoint
        .get("sourceRevision")
        .and_then(Value::as_str)
        .ok_or("review-stale: reviewed revision is missing")?;
    let data: String =
        sqlx::query_scalar("SELECT data FROM spaces_nodes WHERE id=$1 AND deleted_at IS NULL")
            .bind(node_id)
            .fetch_optional(pool)
            .await
            .map_err(db::sql_error)?
            .ok_or("reviewed-revision-changed: reviewed resource is unavailable")?;
    let data: Value = serde_json::from_str(&data).map_err(db::json_error)?;
    let current = data
        .get("knowledgeRevision")
        .or_else(|| data.get("sourceRevision"))
        .and_then(Value::as_str);
    if current != Some(expected) {
        return Err(
            "reviewed-revision-changed: content changed after the review checkpoint".into(),
        );
    }
    Ok(())
}

async fn settle(
    pool: &SqlitePool,
    step_id: &str,
    expected_version: i64,
    reason: &str,
    outcome: Result<&Value, &str>,
) -> Result<(), String> {
    let row = sqlx::query(
        "SELECT run_id,state,state_version,wait_reason FROM spaces_run_steps WHERE id=$1",
    )
    .bind(step_id)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?
    .ok_or("run-step-unavailable: step is unavailable")?;
    let state: String = row.try_get("state").map_err(db::sql_error)?;
    let version: i64 = row.try_get("state_version").map_err(db::sql_error)?;
    let wait_reason: Option<String> = row.try_get("wait_reason").map_err(db::sql_error)?;
    if state != "waiting" || version != expected_version || wait_reason.as_deref() != Some(reason) {
        return Err("run-step-stale: waiting step changed before intervention".into());
    }
    let run_id: String = row.try_get("run_id").map_err(db::sql_error)?;
    scheduler::settle_waiting_step(pool, step_id, &run_id, outcome).await
}
