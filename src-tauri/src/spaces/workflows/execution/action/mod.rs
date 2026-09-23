mod batch;
pub(super) mod batch_items;

use serde_json::{json, Value};
use sqlx::SqlitePool;
use tauri::AppHandle;

use super::{
    iteration,
    shared::{completed, required_str, waiting},
};
use crate::spaces::{
    actions::{self, ActionInvocationOutcome, ActionInvocationRequest},
    runs::{AttemptClaim, StepOutcome, WaitReason},
};

pub(super) async fn execute_run(
    app: &AppHandle,
    pool: &SqlitePool,
    claim: &AttemptClaim,
    input: &Value,
) -> Result<StepOutcome, String> {
    let action_id = required_str(input, "actionId")?;
    let parameters = input
        .get("parameters")
        .ok_or("action-input-invalid: parameters are required")?;
    let inputs = input.get("inputs").unwrap_or(&Value::Null);
    let output = crate::spaces::action_execution::execute_durable(
        app,
        action_id,
        &claim.run_id,
        &claim.home_space_id,
        parameters,
        inputs,
    )
    .await?;
    let now = crate::clock::unix_time_millis()?;
    let mut tx = crate::spaces::db::begin_write(pool).await?;
    crate::spaces::automation::record_action_event(
        &mut tx,
        &claim.attempt_id,
        &claim.home_space_id,
        action_id,
        &output,
        Some(&claim.run_id),
        now,
    )
    .await?;
    tx.commit().await.map_err(crate::spaces::db::sql_error)?;
    completed(output)
}

pub(super) async fn execute(
    pool: &SqlitePool,
    claim: &AttemptClaim,
    action_id: String,
    action_version: u16,
    parameters: Value,
) -> Result<StepOutcome, String> {
    if isolated_write(claim, &action_id)? {
        return completed(json!({
            "isolated": true, "simulatedAction": action_id,
            "input": claim.resolved_input, "parameters": parameters,
        }));
    }
    if let Some(items) = claim
        .resolved_input
        .get("foreachItems")
        .and_then(Value::as_array)
    {
        return batch::execute(pool, claim, action_id, action_version, parameters, items).await;
    }
    if let Some(specification) = claim.resolved_input.get("iteration") {
        return iteration::execute(
            pool,
            claim,
            action_id,
            action_version,
            parameters,
            specification,
        )
        .await;
    }
    execute_invocation(
        pool,
        claim,
        action_id,
        action_version,
        parameters,
        claim.resolved_input.clone(),
    )
    .await
}

pub(super) async fn execute_invocation(
    pool: &SqlitePool,
    claim: &AttemptClaim,
    action_id: String,
    action_version: u16,
    parameters: Value,
    input_snapshot: Value,
) -> Result<StepOutcome, String> {
    if isolated_write(claim, &action_id)? {
        return completed(json!({
            "isolated": true, "simulatedAction": action_id,
            "input": input_snapshot, "parameters": parameters,
        }));
    }
    let outcome = actions::invoke(
        pool,
        ActionInvocationRequest {
            invocation_id: claim.step_instance_id.clone(),
            idempotency_key: claim.step_instance_id.clone(),
            grant_id: required_str(&claim.policy_snapshot, "grantId")?.to_owned(),
            space_id: claim.home_space_id.clone(),
            action_id,
            action_version,
            parameters,
            input_snapshot,
            expected_revisions: Value::Null,
            approval_id: claim
                .previous_output
                .as_ref()
                .and_then(|value| value.get("approvalId"))
                .and_then(Value::as_str)
                .map(str::to_owned),
            parent_run_id: Some(claim.run_id.clone()),
        },
    )
    .await?;
    match outcome {
        ActionInvocationOutcome::QueryResult { result } => completed(result),
        ActionInvocationOutcome::CommittedReceipt { receipt } => completed(receipt),
        ActionInvocationOutcome::AcceptedRun { run_id } => {
            waiting(WaitReason::Executor, json!({ "childRunId": run_id }))
        }
        ActionInvocationOutcome::RequiresApproval {
            approval_id,
            plan_digest,
        } => waiting(
            WaitReason::Approval,
            json!({
                "approvalId": approval_id, "planDigest": plan_digest,
            }),
        ),
    }
}

fn isolated_write(claim: &AttemptClaim, action_id: &str) -> Result<bool, String> {
    Ok(claim
        .policy_snapshot
        .get("testMode")
        .and_then(Value::as_bool)
        == Some(true)
        && actions::require(action_id)?.impact != actions::SpaceActionImpact::ReadOnly)
}
