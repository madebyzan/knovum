use serde_json::{json, Value};
use sqlx::SqlitePool;

use super::super::identity::ensure_active;
use crate::spaces::{
    actions::{self, ActionInvocationOutcome, ActionInvocationRequest},
    db,
    runs::{transition_run, RunState, WaitReason},
};

pub(super) async fn settle(
    pool: &SqlitePool,
    parent_run_id: &str,
    invocation: &mut ActionInvocationRequest,
    mut outcome: ActionInvocationOutcome,
) -> Result<Value, String> {
    loop {
        match outcome {
            ActionInvocationOutcome::AcceptedRun { ref run_id } => {
                set_waiting(pool, parent_run_id, WaitReason::Executor).await?;
                let value = await_child(pool, parent_run_id, run_id).await?;
                set_running(pool, parent_run_id).await?;
                return Ok(value);
            }
            ActionInvocationOutcome::RequiresApproval {
                ref approval_id, ..
            } => {
                set_waiting(pool, parent_run_id, WaitReason::Approval).await?;
                await_approval(pool, parent_run_id, approval_id).await?;
                set_running(pool, parent_run_id).await?;
                invocation.approval_id = Some(approval_id.clone());
                outcome = actions::invoke(pool, invocation.clone()).await?;
            }
            _ => return serde_json::to_value(outcome).map_err(db::json_error),
        }
    }
}

async fn await_child(
    pool: &SqlitePool,
    parent_run_id: &str,
    child_run_id: &str,
) -> Result<Value, String> {
    let mut changes = crate::spaces::runs::subscribe_to_changes();
    loop {
        ensure_active(pool, parent_run_id).await?;
        let run = crate::spaces::runs::load_run(pool, child_run_id).await?;
        if run.state.is_terminal() {
            if !matches!(
                run.state,
                RunState::Succeeded | RunState::CompletedWithErrors
            ) {
                return Err(format!("assistant-child-run-{}", state_text(run.state)));
            }
            let output = sqlx::query_scalar::<_, Option<String>>(
                "SELECT output FROM spaces_run_steps WHERE run_id=$1 AND state='succeeded' ORDER BY updated_at DESC LIMIT 1",
            ).bind(child_run_id).fetch_optional(pool).await.map_err(db::sql_error)?.flatten()
                .map(|encoded| serde_json::from_str(&encoded).map_err(db::json_error)).transpose()?
                .unwrap_or(Value::Null);
            let node_id: Option<String> = sqlx::query_scalar(
                "SELECT output_node_id FROM spaces_knowledge_derivations WHERE id=$1 AND status='completed'",
            )
            .bind(child_run_id)
            .fetch_optional(pool)
            .await
            .map_err(db::sql_error)?;
            return Ok(json!({"type": "accepted_run", "runId": child_run_id,
                "state": state_text(run.state), "output": output, "nodeId": node_id}));
        }
        changes
            .changed()
            .await
            .map_err(|_| "run-change-signal-closed".to_owned())?;
    }
}

async fn await_approval(
    pool: &SqlitePool,
    parent_run_id: &str,
    approval_id: &str,
) -> Result<(), String> {
    let mut changes = crate::spaces::runs::subscribe_to_changes();
    loop {
        ensure_active(pool, parent_run_id).await?;
        let decision: Option<String> =
            sqlx::query_scalar("SELECT decision FROM spaces_action_approvals WHERE id=$1")
                .bind(approval_id)
                .fetch_optional(pool)
                .await
                .map_err(db::sql_error)?;
        match decision.as_deref() {
            Some("approved") => return Ok(()),
            Some("rejected") => {
                return Err("approval-rejected: Assistant action was rejected".into())
            }
            Some("pending") => changes
                .changed()
                .await
                .map_err(|_| "run-change-signal-closed".to_owned())?,
            _ => return Err("approval-stale: Assistant approval is unavailable".into()),
        }
    }
}

async fn set_waiting(pool: &SqlitePool, run_id: &str, reason: WaitReason) -> Result<(), String> {
    let run = crate::spaces::runs::load_run(pool, run_id).await?;
    transition_run(
        pool,
        run_id,
        run.state_version,
        RunState::Waiting,
        Some(reason),
    )
    .await?;
    Ok(())
}

async fn set_running(pool: &SqlitePool, run_id: &str) -> Result<(), String> {
    let run = crate::spaces::runs::load_run(pool, run_id).await?;
    transition_run(pool, run_id, run.state_version, RunState::Running, None).await?;
    Ok(())
}

fn state_text(state: RunState) -> &'static str {
    match state {
        RunState::Queued => "queued",
        RunState::Running => "running",
        RunState::Waiting => "waiting",
        RunState::Paused => "paused",
        RunState::Succeeded => "succeeded",
        RunState::CompletedWithErrors => "completed_with_errors",
        RunState::Failed => "failed",
        RunState::Cancelled => "cancelled",
    }
}
