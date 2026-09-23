use sqlx::{Row, SqlitePool};

use super::ReconciliationDecision;
use crate::spaces::{db, runs::RunEventKind};

pub(crate) async fn reconcile(
    pool: &SqlitePool,
    step_id: &str,
    expected_version: i64,
    decision: ReconciliationDecision,
) -> Result<(), String> {
    let now = crate::clock::unix_time_millis()?;
    let mut tx = db::begin_write(pool).await?;
    let row = sqlx::query(
        "SELECT run_id FROM spaces_run_steps WHERE id=$1 AND state='blocked'
         AND wait_reason='external_reconciliation' AND state_version=$2",
    )
    .bind(step_id)
    .bind(expected_version)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db::sql_error)?
    .ok_or("run-step-stale: reconciliation checkpoint changed")?;
    let run_id: String = row.try_get("run_id").map_err(db::sql_error)?;
    let state = match decision {
        ReconciliationDecision::Retry => "ready",
        ReconciliationDecision::Fail => "failed",
    };
    sqlx::query(
        "UPDATE spaces_run_steps SET state=$1,wait_reason=NULL,state_version=state_version+1,
         available_at=$2,updated_at=$2 WHERE id=$3",
    )
    .bind(state)
    .bind(now)
    .bind(step_id)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    if matches!(decision, ReconciliationDecision::Fail) {
        sqlx::query("UPDATE spaces_run_steps SET state='blocked',wait_reason=NULL,state_version=state_version+1,
             updated_at=$1 WHERE run_id=$2 AND id<>$3 AND state IN ('pending','ready','waiting','paused')")
            .bind(now).bind(&run_id).bind(step_id).execute(&mut *tx).await.map_err(db::sql_error)?;
    }
    let run_state = match decision {
        ReconciliationDecision::Retry => "queued",
        ReconciliationDecision::Fail => "failed",
    };
    let completed_at = matches!(decision, ReconciliationDecision::Fail).then_some(now);
    sqlx::query(
        "UPDATE spaces_runs SET state=$1,wait_reason=NULL,state_version=state_version+1,
         updated_at=$2,completed_at=$3 WHERE id=$4 AND completed_at IS NULL",
    )
    .bind(run_state)
    .bind(now)
    .bind(completed_at)
    .bind(&run_id)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    super::super::append_event(
        &mut tx,
        &run_id,
        Some(step_id),
        None,
        RunEventKind::ExternalOutcomeReconciled,
        &serde_json::json!({"decision": state}),
        now,
    )
    .await?;
    tx.commit().await.map_err(db::sql_error)
}
