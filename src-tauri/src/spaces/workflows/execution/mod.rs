mod action;
mod shared;

use serde_json::json;
use sqlx::SqlitePool;
use tauri::AppHandle;

use crate::spaces::runs::{AttemptClaim, StepOutcome};

pub(crate) async fn execute_step(
    app: &AppHandle,
    pool: &SqlitePool,
    claim: &AttemptClaim,
) -> Result<StepOutcome, String> {
    if claim.node_kind == "captured_action" {
        return super::capture::execute_step(pool, claim).await;
    }
    if claim.node_kind == "action" && claim.configuration == json!({}) {
        return action::execute_run(app, claim, &claim.resolved_input).await;
    }
    Err(format!(
        "workflow-step-invalid: unsupported execution node {}",
        claim.node_kind
    ))
}

pub(crate) async fn invoke_captured(
    pool: &SqlitePool,
    claim: &AttemptClaim,
    action_id: String,
    action_version: u16,
    parameters: serde_json::Value,
    input: serde_json::Value,
) -> Result<StepOutcome, String> {
    action::execute_invocation(pool, claim, action_id, action_version, parameters, input).await
}
