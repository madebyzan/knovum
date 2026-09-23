use serde_json::{json, Value};
use sqlx::SqlitePool;

use super::shared::{completed, required_str, waiting};
use crate::spaces::{
    actions::{self, ActionInvocationOutcome, ActionInvocationRequest},
    runs::{AttemptClaim, StepOutcome, WaitReason},
};

pub(super) async fn execute(
    pool: &SqlitePool,
    claim: &AttemptClaim,
    action_id: String,
    action_version: u16,
    parameters: Value,
    specification: &Value,
) -> Result<StepOutcome, String> {
    let max_iterations = specification
        .get("maxIterations")
        .and_then(Value::as_u64)
        .and_then(|value| u16::try_from(value).ok())
        .ok_or("iterate-input-invalid: maxIterations is missing")?;
    let stop_expression = required_str(specification, "stopExpression")?;
    let mut state = resumed_state(claim)
        .or_else(|| specification.get("state").cloned())
        .ok_or("iterate-input-invalid: state is missing")?;
    let first_index = claim
        .previous_output
        .as_ref()
        .and_then(|value| value.get("iterationIndex"))
        .and_then(Value::as_u64)
        .and_then(|value| u16::try_from(value).ok())
        .unwrap_or(0);
    for index in first_index..max_iterations {
        if super::super::expressions::evaluate(stop_expression, &state)?.as_bool() == Some(true) {
            return completed(json!({ "iterations": index, "state": state }));
        }
        let approved_retry = claim.previous_output.as_ref().is_some_and(|checkpoint| {
            checkpoint.get("iterationIndex").and_then(Value::as_u64) == Some(u64::from(index))
                && checkpoint.get("approvalId").is_some()
        });
        if !approved_retry {
            crate::spaces::runs::reserve_expanded_steps(pool, &claim.run_id, 1).await?;
        }
        let invocation_id = format!("{}:iteration:{index}", claim.step_instance_id);
        let outcome = actions::invoke(
            pool,
            ActionInvocationRequest {
                invocation_id: invocation_id.clone(),
                idempotency_key: invocation_id,
                grant_id: required_str(&claim.policy_snapshot, "grantId")?.to_owned(),
                space_id: claim.home_space_id.clone(),
                action_id: action_id.clone(),
                action_version,
                parameters: parameters.clone(),
                input_snapshot: state.clone(),
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
            ActionInvocationOutcome::QueryResult { result } => state = result,
            ActionInvocationOutcome::CommittedReceipt { receipt } => state = receipt,
            ActionInvocationOutcome::AcceptedRun { run_id } => {
                return waiting(
                    WaitReason::Executor,
                    json!({
                        "childRunId": run_id,
                        "resumeStep": true,
                        "iterationIndex": index + 1,
                        "iterationState": state,
                    }),
                );
            }
            ActionInvocationOutcome::RequiresApproval {
                approval_id,
                plan_digest,
            } => {
                return waiting(
                    WaitReason::Approval,
                    json!({
                        "approvalId": approval_id,
                        "planDigest": plan_digest,
                        "iterationIndex": index,
                        "iterationState": state,
                    }),
                );
            }
        }
    }
    Err("iterate-limit-exceeded: stop expression remained false".into())
}

fn resumed_state(claim: &AttemptClaim) -> Option<Value> {
    let checkpoint = claim.previous_output.as_ref()?;
    checkpoint
        .get("childResult")
        .and_then(|value| value.get("output"))
        .cloned()
        .or_else(|| checkpoint.get("iterationState").cloned())
}
