use crate::foundation::app_error::AppError;
use serde_json::Value;
use sqlx::{Sqlite, SqlitePool, Transaction};

use super::resolution::{
    failure_policy, resolve_dependents, resolve_run, skip_unselected_branches,
};
use crate::spaces::{
    db,
    runs::{run_rows::enum_text, AttemptClaim, WaitReason},
};

mod resumption;
#[cfg(test)]
mod tests;

pub(crate) use resumption::{settle_waiting_step, settle_waiting_step_in_transaction};

pub(crate) async fn complete_step(
    pool: &SqlitePool,
    claim: &AttemptClaim,
    output: &Value,
    selected_branch: Option<&str>,
) -> Result<(), AppError> {
    let now = crate::foundation::clock::unix_time_millis()?;
    let output_bytes = serde_json::to_vec(output).map_err(db::json_error)?.len();
    let mut tx = db::begin_write(pool).await?;
    require_current_claim(&mut tx, claim).await?;
    crate::spaces::runs::budget::reserve_output_bytes_in_transaction(
        &mut tx,
        &claim.run_id,
        output_bytes,
        now,
    )
    .await?;
    finish_attempt(&mut tx, claim, "succeeded", Some(output), None, now).await?;
    update_step(&mut tx, claim, "succeeded", Some(output), None, now).await?;
    if let Some(branch) = selected_branch {
        skip_unselected_branches(&mut tx, claim, branch, now).await?;
    }
    resolve_dependents(&mut tx, &claim.run_id, now).await?;
    resolve_run(&mut tx, &claim.run_id, now).await?;
    Ok(tx.commit().await.map_err(db::sql_error)?)
}

pub(crate) async fn wait_step(
    pool: &SqlitePool,
    claim: &AttemptClaim,
    reason: WaitReason,
    checkpoint: &Value,
) -> Result<(), AppError> {
    let now = crate::foundation::clock::unix_time_millis()?;
    let mut tx = db::begin_write(pool).await?;
    require_current_claim(&mut tx, claim).await?;
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
    let changed = sqlx::query(
        "UPDATE spaces_runs SET state='waiting',state_version=state_version+1,
        wait_reason=$1,updated_at=$2 WHERE id=$3 AND execution_epoch=$4
        AND cancel_requested_at IS NULL AND completed_at IS NULL",
    )
    .bind(enum_text(reason)?)
    .bind(now)
    .bind(&claim.run_id)
    .bind(claim.execution_epoch)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    if changed.rows_affected() != 1 {
        return Err(AppError::new(
            "attempt_stale",
            "Run authority changed before waiting",
        ));
    }

    Ok(tx.commit().await.map_err(db::sql_error)?)
}

pub(crate) async fn fail_step(
    pool: &SqlitePool,
    claim: &AttemptClaim,
    error: &AppError,
) -> Result<(), AppError> {
    let now = crate::foundation::clock::unix_time_millis()?;
    let mut tx = db::begin_write(pool).await?;
    require_current_claim(&mut tx, claim).await?;
    finish_attempt(&mut tx, claim, "failed", None, Some(&error.code), now).await?;
    let max_attempts = claim
        .budget
        .get("maxAttempts")
        .and_then(Value::as_i64)
        .unwrap_or(3)
        .clamp(1, 10);
    if claim.attempt_number < max_attempts && retryable(error) {
        let retry_index = u32::try_from(claim.attempt_number - 1)
            .map_err(|_| AppError::new("attempt_invalid", "Run attempt number is invalid"))?;
        let delay_seconds = 1_i64 << retry_index.min(6);
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
    Ok(tx.commit().await.map_err(db::sql_error)?)
}

async fn require_current_claim(
    tx: &mut Transaction<'_, Sqlite>,
    claim: &AttemptClaim,
) -> Result<(), AppError> {
    let authority: Option<i64> = sqlx::query_scalar(
        "SELECT 1 FROM spaces_run_attempts a
         JOIN spaces_run_steps s ON s.id=a.step_instance_id AND s.run_id=a.run_id
         JOIN spaces_runs r ON r.id=a.run_id
         WHERE a.id=$1 AND a.run_id=$2 AND a.step_instance_id=$3 AND a.state='executing'
         AND a.execution_epoch=$4 AND s.state='running' AND r.state IN ('queued','running','waiting')
         AND r.execution_epoch=$4 AND r.cancel_requested_at IS NULL AND r.completed_at IS NULL",
    )
    .bind(&claim.attempt_id)
    .bind(&claim.run_id)
    .bind(&claim.step_instance_id)
    .bind(claim.execution_epoch)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    authority
        .map(|_| ())
        .ok_or_else(|| AppError::new("attempt_stale", "Run authority changed during execution"))
}

fn retryable(error: &AppError) -> bool {
    matches!(
        error.code.as_str(),
        "transient_failure" | "rate_limited" | "request_timeout"
    )
}

async fn schedule_retry(
    tx: &mut Transaction<'_, Sqlite>,
    claim: &AttemptClaim,
    available_at: i64,
    now: i64,
) -> Result<(), AppError> {
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
    if changed.rows_affected() != 1 {
        return Err(AppError::new(
            "step_stale",
            "Step state changed during retry scheduling",
        ));
    }
    Ok(())
}

async fn finish_attempt(
    tx: &mut Transaction<'_, Sqlite>,
    claim: &AttemptClaim,
    state: &str,
    attempt_output: Option<&Value>,
    error: Option<&str>,
    now: i64,
) -> Result<(), AppError> {
    let changed = sqlx::query("UPDATE spaces_run_attempts SET state=$1,output=$2,error_code=$3,lease_until=NULL,finished_at=$4 WHERE id=$5 AND state='executing' AND execution_epoch=$6")
        .bind(state).bind(attempt_output.map(sqlx::types::Json)).bind(error).bind(now).bind(&claim.attempt_id).bind(claim.execution_epoch)
        .execute(&mut **tx).await.map_err(db::sql_error)?;
    if changed.rows_affected() != 1 {
        return Err(AppError::new(
            "attempt_stale",
            "execution lease is no longer active",
        ));
    }
    Ok(())
}

async fn update_step(
    tx: &mut Transaction<'_, Sqlite>,
    claim: &AttemptClaim,
    state: &str,
    output: Option<&Value>,
    reason: Option<WaitReason>,
    now: i64,
) -> Result<(), AppError> {
    let changed = sqlx::query("UPDATE spaces_run_steps SET state=$1,state_version=state_version+1,output=$2,wait_reason=$3,updated_at=$4 WHERE id=$5 AND state='running'")
        .bind(state).bind(output.map(sqlx::types::Json)).bind(reason.map(enum_text).transpose()?).bind(now)
        .bind(&claim.step_instance_id).execute(&mut **tx).await.map_err(db::sql_error)?;
    if changed.rows_affected() != 1 {
        return Err(AppError::new(
            "step_stale",
            "Step state changed during execution",
        ));
    }
    Ok(())
}
