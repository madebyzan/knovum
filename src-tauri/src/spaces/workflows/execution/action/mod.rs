use crate::app_error::AppError;
use serde_json::{json, Value};
use sqlx::SqlitePool;

use super::shared::{completed, required_str, waiting};
use crate::spaces::{
    actions::{self, ActionInvocationOutcome, ActionInvocationRequest},
    runs::{AttemptClaim, StepOutcome, WaitReason},
};

pub(super) async fn execute_invocation(
    pool: &SqlitePool,
    claim: &AttemptClaim,
    action_id: String,
    action_version: u16,
    parameters: Value,
    input_snapshot: Value,
) -> Result<StepOutcome, AppError> {
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
            parent_run_id: Some(claim.run_id.clone()),
        },
    )
    .await?;
    match outcome {
        ActionInvocationOutcome::QueryResult {
            result: action_output,
        } => Ok(completed(action_output)?),
        ActionInvocationOutcome::CommittedReceipt { receipt } => Ok(completed(receipt)?),
        ActionInvocationOutcome::AcceptedRun { run_id } => Ok(waiting(
            WaitReason::Executor,
            json!({ "childRunId": run_id }),
        )?),
    }
}
