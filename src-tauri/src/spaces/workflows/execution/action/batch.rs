use futures::{stream, StreamExt};
use serde_json::{json, Value};
use sqlx::SqlitePool;

use crate::spaces::{
    actions::{self, ActionInvocationOutcome, ActionInvocationRequest, SpaceActionExecutionMode},
    runs::{AttemptClaim, StepOutcome, WaitReason},
};

use super::{
    super::shared::{completed, required_str, waiting},
    batch_items,
};

pub(super) async fn execute(
    pool: &SqlitePool,
    claim: &AttemptClaim,
    action_id: String,
    action_version: u16,
    parameters: Value,
    items: &[Value],
) -> Result<StepOutcome, String> {
    let parallelism = claim
        .resolved_input
        .get("parallelism")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(1)
        .clamp(1, 64);
    let grant_id = required_str(&claim.policy_snapshot, "grantId")?.to_owned();
    let durable = matches!(
        actions::require(&action_id)?.execution_mode,
        SpaceActionExecutionMode::Durable,
    );
    let retry_root = claim
        .policy_snapshot
        .get("retryRootRunId")
        .and_then(Value::as_str)
        .unwrap_or(&claim.run_id);
    let requests = items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let item_key = required_str(item, "itemKey")?.to_owned();
            let value = item
                .get("value")
                .cloned()
                .ok_or("foreach-item-invalid: value is required")?;
            let invocation_id = format!("{}:{item_key}", claim.step_instance_id);
            let idempotency_key = if durable {
                invocation_id.clone()
            } else {
                format!(
                    "{retry_root}:{}:{item_key}",
                    claim.configuration["id"]
                        .as_str()
                        .unwrap_or(&claim.step_instance_id)
                )
            };
            Ok((
                index,
                item_key.clone(),
                ActionInvocationRequest {
                    invocation_id: invocation_id.clone(),
                    idempotency_key,
                    grant_id: grant_id.clone(),
                    space_id: claim.home_space_id.clone(),
                    action_id: action_id.clone(),
                    action_version,
                    parameters: parameters.clone(),
                    input_snapshot: value,
                    expected_revisions: Value::Null,
                    approval_id: batch_approval(claim.previous_output.as_ref(), &item_key),
                    parent_run_id: Some(claim.run_id.clone()),
                },
            ))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let mut results = stream::iter(
        requests
            .into_iter()
            .map(|(index, key, request)| async move {
                let outcome =
                    match batch_items::prepare(pool, claim, &key, &request.input_snapshot).await {
                        Ok(Some(outcome)) => Ok(outcome),
                        Ok(None) => actions::invoke(pool, request).await,
                        Err(error) => Err(error),
                    };
                batch_items::finish(pool, claim, &key, &outcome).await?;
                Ok::<_, String>((index, key, outcome))
            }),
    )
    .buffer_unordered(parallelism)
    .collect::<Vec<_>>()
    .await
    .into_iter()
    .collect::<Result<Vec<_>, _>>()?;
    results.sort_by_key(|(index, _, _)| *index);
    collect(claim, results)
}

fn collect(
    claim: &AttemptClaim,
    results: Vec<(usize, String, Result<ActionInvocationOutcome, String>)>,
) -> Result<StepOutcome, String> {
    let collect_errors = claim
        .resolved_input
        .get("errorPolicy")
        .and_then(Value::as_str)
        == Some("collect_errors");
    let mut completed_items = Vec::new();
    let mut child_runs = Vec::new();
    let mut approvals = Vec::new();
    for (_, item_key, result) in results {
        let outcome = match result {
            Ok(outcome) => outcome,
            Err(error) if collect_errors => {
                completed_items.push(json!({ "itemKey": item_key, "error": error }));
                continue;
            }
            Err(error) => return Err(error),
        };
        match outcome {
            ActionInvocationOutcome::QueryResult { result } => {
                completed_items.push(json!({ "itemKey": item_key, "result": result }));
            }
            ActionInvocationOutcome::CommittedReceipt { receipt } => {
                completed_items.push(json!({ "itemKey": item_key, "result": receipt }));
            }
            ActionInvocationOutcome::AcceptedRun { run_id } => {
                child_runs.push(json!({ "itemKey": item_key, "childRunId": run_id }));
            }
            ActionInvocationOutcome::RequiresApproval {
                approval_id,
                plan_digest,
            } => {
                approvals.push(json!({
                    "itemKey": item_key, "approvalId": approval_id, "planDigest": plan_digest,
                }));
            }
        }
    }
    if !approvals.is_empty() {
        return waiting(WaitReason::Approval, json!({ "approvals": approvals }));
    }
    if child_runs.is_empty() {
        return completed(Value::Array(completed_items));
    }
    waiting(
        WaitReason::Executor,
        json!({
            "childRuns": child_runs,
            "completedItems": completed_items,
            "errorPolicy": claim.resolved_input.get("errorPolicy")
                .and_then(Value::as_str).unwrap_or("stop"),
        }),
    )
}

fn batch_approval(checkpoint: Option<&Value>, item_key: &str) -> Option<String> {
    checkpoint?
        .get("approvals")?
        .as_array()?
        .iter()
        .find_map(|approval| {
            (approval.get("itemKey").and_then(Value::as_str) == Some(item_key))
                .then(|| {
                    approval
                        .get("approvalId")
                        .and_then(Value::as_str)
                        .map(str::to_owned)
                })
                .flatten()
        })
}
