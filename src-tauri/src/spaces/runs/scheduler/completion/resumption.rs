use serde_json::Value;
use sqlx::SqlitePool;

use super::{resolve_dependents, resolve_run};
use crate::spaces::{db, runs::RunEventKind};

pub(crate) async fn settle_waiting_step(
    pool: &SqlitePool,
    step_id: &str,
    run_id: &str,
    outcome: Result<&Value, &str>,
) -> Result<(), String> {
    let now = crate::clock::unix_time_millis()?;
    let mut tx = db::begin_write(pool).await?;
    match outcome {
        Ok(output) => {
            sqlx::query("UPDATE spaces_run_steps SET state='succeeded',output=$1,wait_reason=NULL,state_version=state_version+1,updated_at=$2 WHERE id=$3 AND state='waiting'")
                .bind(serde_json::to_string(output).map_err(db::json_error)?)
                .bind(now).bind(step_id).execute(&mut *tx).await.map_err(db::sql_error)?;
        }
        Err(error) => {
            sqlx::query("UPDATE spaces_run_steps SET state='failed',wait_reason=NULL,state_version=state_version+1,updated_at=$1 WHERE id=$2 AND state='waiting'")
                .bind(now).bind(step_id).execute(&mut *tx).await.map_err(db::sql_error)?;
            super::super::super::append_event(
                &mut tx,
                run_id,
                Some(step_id),
                None,
                RunEventKind::StepFailed,
                &serde_json::json!({"error": error}),
                now,
            )
            .await?;
        }
    }
    resolve_dependents(&mut tx, run_id, now).await?;
    sqlx::query("UPDATE spaces_runs SET state='queued',wait_reason=NULL,state_version=state_version+1,updated_at=$1 WHERE id=$2 AND state='waiting'")
        .bind(now).bind(run_id).execute(&mut *tx).await.map_err(db::sql_error)?;
    resolve_run(&mut tx, run_id, now).await?;
    tx.commit().await.map_err(db::sql_error)
}

pub(crate) async fn resume_approved_step(
    pool: &SqlitePool,
    step_id: &str,
    run_id: &str,
) -> Result<(), String> {
    let now = crate::clock::unix_time_millis()?;
    let mut tx = db::begin_write(pool).await?;
    let changed = sqlx::query("UPDATE spaces_run_steps SET state='ready',wait_reason=NULL,state_version=state_version+1,updated_at=$1 WHERE id=$2 AND state='waiting' AND wait_reason='approval'")
        .bind(now).bind(step_id).execute(&mut *tx).await.map_err(db::sql_error)?;
    db::require_one(
        changed,
        "run-step-stale: approval step changed before resumption",
    )?;
    queue_run(&mut tx, run_id, now).await?;
    tx.commit().await.map_err(db::sql_error)
}

pub(crate) async fn resume_step_with_output(
    pool: &SqlitePool,
    step_id: &str,
    run_id: &str,
    output: &Value,
) -> Result<(), String> {
    let now = crate::clock::unix_time_millis()?;
    let encoded = serde_json::to_string(output).map_err(db::json_error)?;
    let mut tx = db::begin_write(pool).await?;
    let changed = sqlx::query(
        "UPDATE spaces_run_steps SET state='ready',output=$1,wait_reason=NULL,
         state_version=state_version+1,updated_at=$2 WHERE id=$3 AND state='waiting'",
    )
    .bind(encoded)
    .bind(now)
    .bind(step_id)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    db::require_one(
        changed,
        "run-step-stale: waiting step changed before resumption",
    )?;
    queue_run(&mut tx, run_id, now).await?;
    tx.commit().await.map_err(db::sql_error)
}

async fn queue_run(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    run_id: &str,
    now: i64,
) -> Result<(), String> {
    sqlx::query("UPDATE spaces_runs SET state='queued',wait_reason=NULL,state_version=state_version+1,updated_at=$1 WHERE id=$2 AND state='waiting'")
        .bind(now).bind(run_id).execute(&mut **tx).await.map_err(db::sql_error)?;
    Ok(())
}
