mod action;
mod control;
mod delegate;
mod iteration;
mod publish;
mod shared;
mod subworkflow_batch;
#[cfg(test)]
mod tests;

use serde_json::json;
use sqlx::SqlitePool;
use tauri::AppHandle;

use super::definition::WorkflowNode;
use crate::spaces::runs::{AttemptClaim, StepOutcome, WaitReason};

pub(crate) async fn execute_step(
    app: &AppHandle,
    pool: &SqlitePool,
    claim: &AttemptClaim,
) -> Result<StepOutcome, String> {
    if claim.node_kind == "action" && claim.configuration == json!({}) {
        return action::execute_run(app, pool, claim, &claim.resolved_input).await;
    }
    let node: WorkflowNode = serde_json::from_value(claim.configuration.clone())
        .map_err(crate::spaces::db::json_error)?;
    match node {
        WorkflowNode::Input { .. }
        | WorkflowNode::Output { .. }
        | WorkflowNode::Parallel { .. }
        | WorkflowNode::Join { .. } => shared::completed(claim.resolved_input.clone()),
        WorkflowNode::Action {
            action_id,
            action_version,
            parameters,
            ..
        } => action::execute(pool, claim, action_id, action_version, parameters).await,
        WorkflowNode::Condition { expression, .. } => {
            control::condition(&expression, &claim.resolved_input)
        }
        WorkflowNode::ForEach {
            max_items,
            parallelism,
            error_policy,
            ..
        } => control::for_each(&claim.resolved_input, max_items, parallelism, error_policy),
        WorkflowNode::Iterate {
            max_iterations,
            stop_expression,
            ..
        } => control::iterate(
            claim.resolved_input.clone(),
            stop_expression,
            max_iterations,
        ),
        WorkflowNode::Subworkflow {
            workflow_id,
            version,
            ..
        } => control::start_subworkflow(pool, claim, workflow_id, version).await,
        WorkflowNode::HumanInput { schema, .. } => {
            shared::waiting(WaitReason::UserInput, json!({ "schema": schema }))
        }
        WorkflowNode::Review { purpose, .. } => control::review(&claim.resolved_input, purpose),
        WorkflowNode::Delay {
            duration_seconds, ..
        } => control::delay(claim, duration_seconds),
        WorkflowNode::DataTransform { expression, .. } => shared::completed(
            super::expressions::evaluate(&expression, &claim.resolved_input)?,
        ),
        WorkflowNode::ExternalTool {
            binding_id,
            schema_fingerprint,
            ..
        } => super::external_tools::execute(pool, claim, &binding_id, &schema_fingerprint).await,
        WorkflowNode::DelegateTask { runtime_id, .. } => {
            delegate::execute(app, claim, &runtime_id).await
        }
        WorkflowNode::Publish {
            mode, destination, ..
        } => publish::execute(app, pool, claim, mode, destination.as_deref()).await,
    }
}
