mod action;
mod review;
mod shared;

use serde_json::json;
use sqlx::SqlitePool;
use tauri::AppHandle;

use super::definition::WorkflowNode;
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
    let node: WorkflowNode = serde_json::from_value(claim.configuration.clone())
        .map_err(crate::spaces::db::json_error)?;
    match node {
        WorkflowNode::Input { .. } | WorkflowNode::Output { .. } | WorkflowNode::Join { .. } => {
            shared::completed(claim.resolved_input.clone())
        }
        WorkflowNode::Action {
            action_id,
            action_version,
            parameters,
            ..
        } => action::execute(pool, claim, action_id, action_version, parameters).await,
        WorkflowNode::Review { purpose, .. } => {
            review::execute(pool, &claim.home_space_id, &claim.resolved_input, purpose).await
        }
    }
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
