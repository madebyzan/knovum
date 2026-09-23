use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;

use super::identity::TaskIds;
use crate::spaces::{
    authorization::{consume_approval, create_approval, ApprovalRequest},
    db,
    runs::{transition_run, RunState, WaitReason},
    workflows::external_tools::{ExternalToolBinding, ExternalToolEffect},
};

pub(super) async fn approve_if_needed(
    pool: &SqlitePool,
    ids: &TaskIds,
    tool: &ExternalToolBinding,
    arguments: &Value,
) -> Result<(), String> {
    if matches!(tool.effect, ExternalToolEffect::ReadOnly) {
        return Ok(());
    }
    let action_id = format!("external-tool:{}", tool.id);
    let encoded = serde_json::to_vec(&json!({"binding": tool.id, "arguments": arguments}))
        .map_err(db::json_error)?;
    let digest = format!("{:x}", Sha256::digest(encoded));
    let now = crate::clock::unix_time_millis()?;
    let approval_id = uuid::Uuid::new_v4().to_string();
    let mut tx = db::begin_write(pool).await?;
    create_approval(
        &mut tx,
        &ApprovalRequest {
            id: approval_id.clone(),
            grant_id: ids.grant_id.clone(),
            space_id: tool.space_id.clone(),
            action_id: action_id.clone(),
            plan_digest: digest.clone(),
            plan: json!({
                "bindingId": tool.id,
                "name": tool.name,
                "endpoint": tool.endpoint,
                "toolName": tool.tool_name,
                "effect": tool.effect,
                "arguments": arguments,
            }),
            expected_revisions: Value::Null,
            valid_until: now + 15 * 60 * 1_000,
        },
        now,
    )
    .await?;
    tx.commit().await.map_err(db::sql_error)?;
    transition(
        pool,
        &ids.run_id,
        RunState::Waiting,
        Some(WaitReason::Approval),
    )
    .await?;
    await_approval(pool, &ids.run_id, &approval_id).await?;
    transition(pool, &ids.run_id, RunState::Running, None).await?;
    let mut tx = db::begin_write(pool).await?;
    consume_approval(
        &mut tx,
        &approval_id,
        &action_id,
        &digest,
        crate::clock::unix_time_millis()?,
    )
    .await?;
    tx.commit().await.map_err(db::sql_error)
}

async fn await_approval(pool: &SqlitePool, run_id: &str, approval_id: &str) -> Result<(), String> {
    let mut changes = crate::spaces::runs::subscribe_to_changes();
    loop {
        let active: Option<i64> = sqlx::query_scalar(
            "SELECT 1 FROM spaces_runs WHERE id=$1 AND cancel_requested_at IS NULL AND completed_at IS NULL",
        ).bind(run_id).fetch_optional(pool).await.map_err(db::sql_error)?;
        if active.is_none() {
            return Err("assistant-task-inactive: task was cancelled".into());
        }
        let decision: Option<String> =
            sqlx::query_scalar("SELECT decision FROM spaces_action_approvals WHERE id=$1")
                .bind(approval_id)
                .fetch_optional(pool)
                .await
                .map_err(db::sql_error)?;
        match decision.as_deref() {
            Some("approved") => return Ok(()),
            Some("rejected") => {
                return Err("approval-rejected: external tool call was rejected".into())
            }
            Some("pending") => changes
                .changed()
                .await
                .map_err(|_| "run-change-signal-closed".to_owned())?,
            _ => return Err("approval-stale: external tool approval is unavailable".into()),
        }
    }
}

async fn transition(
    pool: &SqlitePool,
    run_id: &str,
    state: RunState,
    reason: Option<WaitReason>,
) -> Result<(), String> {
    let run = crate::spaces::runs::load_run(pool, run_id).await?;
    transition_run(pool, run_id, run.state_version, state, reason)
        .await
        .map(|_| ())
}
