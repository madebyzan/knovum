use serde_json::Value;
use sqlx::{Row, Sqlite, SqlitePool, Transaction};

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

pub(super) async fn decide_review(
    pool: &SqlitePool,
    step_id: &str,
    expected_version: i64,
    reviewed_revision: &str,
    approved: bool,
) -> Result<(), String> {
    let checkpoint: Option<String> = sqlx::query_scalar(
        "SELECT output FROM spaces_run_steps WHERE id=$1 AND state='waiting' AND wait_reason='content_review'",
    )
    .bind(step_id)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?
    .flatten();
    let checkpoint: Value = serde_json::from_str(
        checkpoint
            .as_deref()
            .ok_or("review-stale: review checkpoint is unavailable")?,
    )
    .map_err(db::json_error)?;
    let node_id = checkpoint
        .get("nodeId")
        .and_then(Value::as_str)
        .ok_or("review-stale: reviewed resource is missing")?;
    let approved_value =
        serde_json::json!({"nodeId": node_id, "approvedRevision": reviewed_revision});
    let outcome = if approved {
        Ok(&approved_value)
    } else {
        Err("content-review-rejected")
    };
    settle(
        pool,
        step_id,
        expected_version,
        "content_review",
        outcome,
        approved.then_some(reviewed_revision),
    )
    .await
}

async fn validate_review_revision(
    tx: &mut Transaction<'_, Sqlite>,
    step_id: &str,
    reviewed_revision: &str,
) -> Result<(), String> {
    if reviewed_revision.is_empty() {
        return Err("review-invalid: reviewed revision is required".into());
    }
    let encoded: String = sqlx::query_scalar::<_, Option<String>>(
        "SELECT output FROM spaces_run_steps WHERE id=$1 AND state='waiting'
         AND wait_reason='content_review'",
    )
    .bind(step_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)?
    .flatten()
    .ok_or("review-stale: review checkpoint is unavailable")?;
    let checkpoint: Value = serde_json::from_str(&encoded).map_err(db::json_error)?;
    let node_id = checkpoint
        .get("nodeId")
        .and_then(Value::as_str)
        .ok_or("review-stale: reviewed resource is missing")?;
    if checkpoint
        .get("sourceRevision")
        .and_then(Value::as_str)
        .is_none()
    {
        return Err("review-stale: original revision is missing".into());
    }
    let data: String = sqlx::query_scalar(
        "SELECT n.data FROM spaces_nodes n
             JOIN spaces_run_steps s ON s.id=$2
             JOIN spaces_runs r ON r.id=s.run_id AND r.home_space_id=n.space_id
             WHERE n.id=$1 AND n.deleted_at IS NULL",
    )
    .bind(node_id)
    .bind(step_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)?
    .ok_or("reviewed-revision-changed: reviewed resource is unavailable")?;
    let data: Value = serde_json::from_str(&data).map_err(db::json_error)?;
    let current = data
        .get("knowledgeRevision")
        .or_else(|| data.get("sourceRevision"))
        .and_then(Value::as_str);
    if current != Some(reviewed_revision) {
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
    reviewed_revision: Option<&str>,
) -> Result<(), String> {
    let mut tx = db::begin_write(pool).await?;
    let row = sqlx::query(
        "SELECT run_id,state,state_version,wait_reason FROM spaces_run_steps WHERE id=$1",
    )
    .bind(step_id)
    .fetch_optional(&mut *tx)
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
    if let Some(revision) = reviewed_revision {
        validate_review_revision(&mut tx, step_id, revision).await?;
    }
    scheduler::settle_waiting_step_in_transaction(&mut tx, step_id, &run_id, outcome).await?;
    tx.commit().await.map_err(db::sql_error)
}
