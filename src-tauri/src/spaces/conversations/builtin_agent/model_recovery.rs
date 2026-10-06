use sqlx::{Sqlite, SqlitePool, Transaction};

use crate::spaces::{db, runs::WaitReason};

pub(super) async fn reason(pool: &SqlitePool, run_id: &str) -> Result<Option<WaitReason>, String> {
    let state: Option<String> = sqlx::query_scalar("SELECT c.state FROM spaces_builtin_agent_model_calls c JOIN spaces_builtin_agent_tasks t ON t.id=c.task_id
        WHERE t.run_id=$1 AND c.state!='committed' AND c.retry_authorized_at IS NULL
        ORDER BY CASE c.state WHEN 'usage_unknown' THEN 0 WHEN 'received' THEN 1 ELSE 2 END LIMIT 1")
        .bind(run_id).fetch_optional(pool).await.map_err(db::sql_error)?;
    Ok(state.map(|state| match state.as_str() {
        "usage_unknown" => WaitReason::ModelUsageUnknown,
        "received" => WaitReason::ModelResponseUncommitted,
        _ => WaitReason::ModelOutcomeUnknown,
    }))
}

pub(super) async fn wait(
    pool: &SqlitePool,
    run_id: &str,
    reason: WaitReason,
) -> Result<(), String> {
    let reason = serde_json::to_value(reason).map_err(db::json_error)?;
    sqlx::query("UPDATE spaces_runs SET state='waiting',wait_reason=$1,state_version=state_version+1,updated_at=$2
        WHERE id=$3 AND completed_at IS NULL AND cancel_requested_at IS NULL")
        .bind(reason.as_str()).bind(crate::clock::unix_time_millis()?).bind(run_id).execute(pool).await.map_err(db::sql_error)?;
    crate::spaces::runs::announce_change();
    Ok(())
}

pub(super) async fn authorize_retry(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: &str,
    expected_version: i64,
    now: i64,
) -> Result<(), String> {
    let updated = sqlx::query("UPDATE spaces_runs SET state='queued',wait_reason=NULL,pause_requested_at=NULL,state_version=state_version+1,updated_at=$1
        WHERE id=$2 AND state_version=$3 AND state='waiting' AND completed_at IS NULL AND cancel_requested_at IS NULL
        AND wait_reason IN ('model_outcome_unknown','model_response_uncommitted','model_usage_unknown')")
        .bind(now).bind(run_id).bind(expected_version).execute(&mut **tx).await.map_err(db::sql_error)?;
    db::require_one(updated, "builtin-agent-recovery-state-changed")?;
    sqlx::query("UPDATE spaces_builtin_agent_model_calls SET retry_authorized_at=$1 WHERE task_id IN (SELECT id FROM spaces_builtin_agent_tasks WHERE run_id=$2)
        AND state!='committed' AND retry_authorized_at IS NULL")
        .bind(now).bind(run_id).execute(&mut **tx).await.map_err(db::sql_error)?;
    crate::spaces::runs::append_event(tx, run_id, None, None, crate::spaces::runs::RunEventKind::BuiltinAgentModelRetryAuthorized,
        &serde_json::json!({"previousStateVersion":expected_version,"possibleDuplicateCharge":true}), now).await?;
    Ok(())
}
