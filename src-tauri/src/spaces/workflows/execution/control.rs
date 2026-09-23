use serde_json::{json, Value};
use sqlx::SqlitePool;

use super::shared::{completed, required_str, waiting};
use crate::spaces::runs::{AttemptClaim, StepOutcome, WaitReason};

pub(super) fn condition(expression: &str, input: &Value) -> Result<StepOutcome, String> {
    let result = super::super::expressions::evaluate(expression, input)?;
    let selected = result
        .as_bool()
        .ok_or("condition-result-invalid: CEL condition must return boolean")?;
    Ok(StepOutcome::Completed {
        output: Value::Bool(selected),
        selected_branch: Some(selected.to_string()),
    })
}

pub(super) fn for_each(
    input: &Value,
    max_items: u32,
    parallelism: u16,
    error_policy: super::super::definition::ItemErrorPolicy,
) -> Result<StepOutcome, String> {
    let items = input
        .as_array()
        .ok_or("foreach-input-invalid: ForEach input must be an array")?;
    if items.len() > max_items as usize {
        return Err("foreach-limit-exceeded: input collection exceeds maxItems".into());
    }
    let mut occurrences = std::collections::HashMap::<String, usize>::new();
    let foreach_items = items
        .iter()
        .map(|item| {
            let base = item
                .get("itemKey")
                .or_else(|| item.get("id"))
                .or_else(|| item.get("key"))
                .and_then(Value::as_str)
                .map(str::to_owned)
                .unwrap_or_else(|| item_digest(item));
            let occurrence = occurrences.entry(base.clone()).or_default();
            let item_key = if *occurrence == 0 {
                base.clone()
            } else {
                format!("{base}:{}", occurrence)
            };
            *occurrence += 1;
            json!({ "itemKey": item_key, "value": item })
        })
        .collect::<Vec<_>>();
    completed(json!({
        "foreachItems": foreach_items,
        "parallelism": parallelism,
        "errorPolicy": error_policy,
    }))
}

fn item_digest(item: &Value) -> String {
    use sha2::Digest;
    let bytes = serde_json::to_vec(item).expect("serde_json::Value always serializes");
    format!("{:x}", sha2::Sha256::digest(bytes))
}

pub(super) fn iterate(
    state: Value,
    stop_expression: String,
    max_iterations: u16,
) -> Result<StepOutcome, String> {
    let stop = super::super::expressions::evaluate(&stop_expression, &state)?;
    if stop.as_bool() == Some(true) {
        return completed(json!({ "iterations": 0, "state": state }));
    }
    completed(json!({ "iteration": {
        "state": state,
        "stopExpression": stop_expression,
        "maxIterations": max_iterations,
    }}))
}

pub(super) fn review(input: &Value, purpose: String) -> Result<StepOutcome, String> {
    let node_id = input
        .get("nodeId")
        .and_then(Value::as_str)
        .ok_or("review-input-invalid: nodeId is required")?;
    let data = input.get("data").unwrap_or(input);
    let revision = data
        .get("knowledgeRevision")
        .or_else(|| data.get("sourceRevision"))
        .and_then(Value::as_str)
        .ok_or("review-input-invalid: a resource revision is required")?;
    waiting(
        WaitReason::ContentReview,
        json!({
            "purpose": purpose,
            "nodeId": node_id,
            "sourceRevision": revision,
        }),
    )
}

pub(super) async fn start_subworkflow(
    pool: &SqlitePool,
    claim: &AttemptClaim,
    workflow_id: String,
    version: u32,
) -> Result<StepOutcome, String> {
    if let Some(items) = claim
        .resolved_input
        .get("foreachItems")
        .and_then(Value::as_array)
    {
        return super::subworkflow_batch::start(pool, claim, workflow_id, version, items).await;
    }
    let run = super::super::runtime::start(
        pool,
        super::super::runtime::StartWorkflowRequest {
            space_id: claim.home_space_id.clone(),
            workflow_id,
            version: i64::from(version),
            input: claim.resolved_input.clone(),
            parent_run_id: Some(claim.run_id.clone()),
            grant_id: Some(required_str(&claim.policy_snapshot, "grantId")?.to_owned()),
            test_mode: claim
                .policy_snapshot
                .get("testMode")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            breakpoint_node_ids: Vec::new(),
        },
    )
    .await?;
    waiting(WaitReason::Executor, json!({ "childRunId": run.id }))
}

pub(super) fn delay(claim: &AttemptClaim, seconds: u64) -> Result<StepOutcome, String> {
    if claim
        .resolved_input
        .get("delayCompleted")
        .and_then(Value::as_bool)
        == Some(true)
    {
        return completed(claim.resolved_input.clone());
    }
    let millis = i64::try_from(seconds)
        .map_err(|_| "delay-invalid: duration is too large")?
        .checked_mul(1_000)
        .ok_or("delay-invalid: duration is too large")?;
    waiting(
        WaitReason::Delay,
        json!({
            "resumeAt": crate::clock::unix_time_millis()? + millis,
        }),
    )
}
