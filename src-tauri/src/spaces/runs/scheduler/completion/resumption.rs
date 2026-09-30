use serde_json::Value;
use sqlx::{Sqlite, SqlitePool, Transaction};

use super::{resolve_dependents, resolve_run};
use crate::spaces::{db, runs::RunEventKind};

pub(crate) async fn settle_waiting_step(
    pool: &SqlitePool,
    step_id: &str,
    run_id: &str,
    outcome: Result<&Value, &str>,
) -> Result<(), String> {
    let mut tx = db::begin_write(pool).await?;
    settle_waiting_step_in_transaction(&mut tx, step_id, run_id, outcome).await?;
    tx.commit().await.map_err(db::sql_error)?;
    super::super::super::signals::announce();
    Ok(())
}

pub(crate) async fn settle_waiting_step_in_transaction(
    tx: &mut Transaction<'_, Sqlite>,
    step_id: &str,
    run_id: &str,
    outcome: Result<&Value, &str>,
) -> Result<(), String> {
    require_active_waiting_step(tx, step_id, run_id).await?;
    let now = crate::clock::unix_time_millis()?;
    match outcome {
        Ok(output) => {
            let changed = sqlx::query("UPDATE spaces_run_steps SET state='succeeded',output=$1,wait_reason=NULL,state_version=state_version+1,updated_at=$2 WHERE id=$3 AND state='waiting'")
                .bind(serde_json::to_string(output).map_err(db::json_error)?)
                .bind(now).bind(step_id).execute(&mut **tx).await.map_err(db::sql_error)?;
            db::require_one(
                changed,
                "run-step-stale: waiting step changed before settlement",
            )?;
        }
        Err(error) => {
            let changed = sqlx::query("UPDATE spaces_run_steps SET state='failed',wait_reason=NULL,state_version=state_version+1,updated_at=$1 WHERE id=$2 AND state='waiting'")
                .bind(now).bind(step_id).execute(&mut **tx).await.map_err(db::sql_error)?;
            db::require_one(
                changed,
                "run-step-stale: waiting step changed before settlement",
            )?;
            super::super::super::append_event(
                tx,
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
    resolve_dependents(tx, run_id, now).await?;
    sqlx::query("UPDATE spaces_runs SET state='queued',wait_reason=NULL,state_version=state_version+1,updated_at=$1 WHERE id=$2 AND state='waiting'")
        .bind(now).bind(run_id).execute(&mut **tx).await.map_err(db::sql_error)?;
    resolve_run(tx, run_id, now).await?;
    Ok(())
}

async fn require_active_waiting_step(
    tx: &mut Transaction<'_, Sqlite>,
    step_id: &str,
    run_id: &str,
) -> Result<(), String> {
    let active: Option<i64> = sqlx::query_scalar(
        "SELECT 1 FROM spaces_run_steps s JOIN spaces_runs r ON r.id=s.run_id
         WHERE s.id=$1 AND s.run_id=$2 AND s.state='waiting'
         AND r.state IN ('queued','running','waiting') AND r.cancel_requested_at IS NULL
         AND r.pause_requested_at IS NULL AND r.completed_at IS NULL",
    )
    .bind(step_id)
    .bind(run_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    active
        .map(|_| ())
        .ok_or_else(|| "run-step-stale: waiting Run authority changed before settlement".into())
}
