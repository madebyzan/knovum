use serde_json::Value;
use sqlx::{Sqlite, SqlitePool, Transaction};

use super::resolution::{
    failure_policy, resolve_dependents, resolve_run, skip_unselected_branches,
};
use crate::spaces::{
    db,
    runs::{AttemptClaim, WaitReason},
};

mod resumption;

pub(crate) use resumption::{resume_approved_step, resume_step_with_output, settle_waiting_step};

pub(crate) async fn complete_step(
    pool: &SqlitePool,
    claim: &AttemptClaim,
    output: &Value,
    selected_branch: Option<&str>,
) -> Result<(), String> {
    let now = crate::clock::unix_time_millis()?;
    let mut tx = db::begin_write(pool).await?;
    finish_attempt(&mut tx, claim, "succeeded", Some(output), None, now).await?;
    update_step(&mut tx, claim, "succeeded", Some(output), None, now).await?;
    if let Some(branch) = selected_branch {
        skip_unselected_branches(&mut tx, claim, branch, now).await?;
    }
    resolve_dependents(&mut tx, &claim.run_id, now).await?;
    resolve_run(&mut tx, &claim.run_id, now).await?;
    tx.commit().await.map_err(db::sql_error)
}

pub(crate) async fn wait_step(
    pool: &SqlitePool,
    claim: &AttemptClaim,
    reason: WaitReason,
    checkpoint: &Value,
) -> Result<(), String> {
    let now = crate::clock::unix_time_millis()?;
    let mut tx = db::begin_write(pool).await?;
    finish_attempt(&mut tx, claim, "succeeded", Some(checkpoint), None, now).await?;
    update_step(
        &mut tx,
        claim,
        "waiting",
        Some(checkpoint),
        Some(reason),
        now,
    )
    .await?;
    sqlx::query("UPDATE spaces_runs SET state='waiting',state_version=state_version+1,wait_reason=$1,updated_at=$2 WHERE id=$3 AND execution_epoch=$4 AND completed_at IS NULL")
        .bind(enum_text(reason)?).bind(now).bind(&claim.run_id).bind(claim.execution_epoch)
        .execute(&mut *tx).await.map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)
}

pub(crate) async fn fail_step(
    pool: &SqlitePool,
    claim: &AttemptClaim,
    error: &str,
) -> Result<(), String> {
    let now = crate::clock::unix_time_millis()?;
    let mut tx = db::begin_write(pool).await?;
    finish_attempt(&mut tx, claim, "failed", None, Some(error), now).await?;
    let max_attempts = claim
        .budget
        .get("maxAttempts")
        .and_then(Value::as_i64)
        .unwrap_or(3)
        .clamp(1, 10);
    if claim.attempt_number < max_attempts && retryable(error) {
        let delay_seconds = 1_i64 << u32::try_from(claim.attempt_number - 1).unwrap_or(0).min(6);
        schedule_retry(&mut tx, claim, now + delay_seconds * 1_000, now).await?;
        super::super::append_event(
            &mut tx,
            &claim.run_id,
            Some(&claim.step_instance_id),
            Some(&claim.attempt_id),
            crate::spaces::runs::RunEventKind::StepRetryScheduled,
            &serde_json::json!({"attempt": claim.attempt_number + 1, "error": error}),
            now,
        )
        .await?;
        tx.commit().await.map_err(db::sql_error)?;
        return Ok(());
    }
    update_step(&mut tx, claim, "failed", None, None, now).await?;
    super::super::append_event(
        &mut tx,
        &claim.run_id,
        Some(&claim.step_instance_id),
        Some(&claim.attempt_id),
        crate::spaces::runs::RunEventKind::StepFailed,
        &serde_json::json!({"error": error}),
        now,
    )
    .await?;
    if failure_policy(&claim.policy_snapshot) == "stop_run" {
        sqlx::query("UPDATE spaces_run_steps SET state='blocked',wait_reason=NULL,state_version=state_version+1,updated_at=$1 WHERE run_id=$2 AND state IN ('pending','ready')")
            .bind(now).bind(&claim.run_id).execute(&mut *tx).await.map_err(db::sql_error)?;
    } else {
        resolve_dependents(&mut tx, &claim.run_id, now).await?;
    }
    resolve_run(&mut tx, &claim.run_id, now).await?;
    tx.commit().await.map_err(db::sql_error)
}

fn retryable(error: &str) -> bool {
    ["transient-failure:", "rate-limited:", "request-timeout:"]
        .iter()
        .any(|prefix| error.starts_with(prefix))
}

async fn schedule_retry(
    tx: &mut Transaction<'_, Sqlite>,
    claim: &AttemptClaim,
    available_at: i64,
    now: i64,
) -> Result<(), String> {
    let changed = sqlx::query(
        "UPDATE spaces_run_steps SET state='ready',state_version=state_version+1,output=NULL,
         wait_reason=NULL,available_at=$1,updated_at=$2 WHERE id=$3 AND state='running'",
    )
    .bind(available_at)
    .bind(now)
    .bind(&claim.step_instance_id)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    db::require_one(
        changed,
        "step-stale: Step state changed during retry scheduling",
    )
}

async fn finish_attempt(
    tx: &mut Transaction<'_, Sqlite>,
    claim: &AttemptClaim,
    state: &str,
    result: Option<&Value>,
    error: Option<&str>,
    now: i64,
) -> Result<(), String> {
    let result = result
        .map(serde_json::to_string)
        .transpose()
        .map_err(db::json_error)?;
    let changed = sqlx::query("UPDATE spaces_run_attempts SET state=$1,result=$2,error_code=$3,lease_until=NULL,finished_at=$4 WHERE id=$5 AND state='executing' AND execution_epoch=$6")
        .bind(state).bind(result).bind(error).bind(now).bind(&claim.attempt_id).bind(claim.execution_epoch)
        .execute(&mut **tx).await.map_err(db::sql_error)?;
    db::require_one(
        changed,
        "attempt-stale: execution lease is no longer active",
    )
}

async fn update_step(
    tx: &mut Transaction<'_, Sqlite>,
    claim: &AttemptClaim,
    state: &str,
    output: Option<&Value>,
    reason: Option<WaitReason>,
    now: i64,
) -> Result<(), String> {
    let output = output
        .map(serde_json::to_string)
        .transpose()
        .map_err(db::json_error)?;
    let changed = sqlx::query("UPDATE spaces_run_steps SET state=$1,state_version=state_version+1,output=$2,wait_reason=$3,updated_at=$4 WHERE id=$5 AND state='running'")
        .bind(state).bind(output).bind(reason.map(enum_text).transpose()?).bind(now)
        .bind(&claim.step_instance_id).execute(&mut **tx).await.map_err(db::sql_error)?;
    db::require_one(changed, "step-stale: Step state changed during execution")
}

fn enum_text(value: impl serde::Serialize) -> Result<String, String> {
    serde_json::to_value(value)
        .map_err(db::json_error)?
        .as_str()
        .map(str::to_owned)
        .ok_or("Invalid enum".into())
}
