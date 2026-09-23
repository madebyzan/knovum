use futures::{stream, StreamExt};
use serde_json::{json, Value};
use sqlx::SqlitePool;

use super::{
    action::batch_items,
    shared::{required_str, waiting},
};
use crate::spaces::{
    runs::{AttemptClaim, StepOutcome, WaitReason},
    workflows::runtime,
};

pub(super) async fn start(
    pool: &SqlitePool,
    claim: &AttemptClaim,
    workflow_id: String,
    version: u32,
    items: &[Value],
) -> Result<StepOutcome, String> {
    let parallelism = claim
        .resolved_input
        .get("parallelism")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(1)
        .clamp(1, 64);
    let requests = items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            Ok((
                index,
                required_str(item, "itemKey")?.to_owned(),
                item.get("value")
                    .cloned()
                    .ok_or("foreach-item-invalid: value is required")?,
            ))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let mut children = stream::iter(requests.into_iter().map(|(index, item_key, input)| {
        let workflow_id = workflow_id.clone();
        async move {
            let existing = batch_items::prepare_child(pool, claim, &item_key, &input).await?;
            let child_id = if let Some(child_id) = existing {
                child_id
            } else {
                let child_id = uuid::Uuid::new_v4().to_string();
                runtime::start_with_run_id(
                    pool,
                    runtime::StartWorkflowRequest {
                        space_id: claim.home_space_id.clone(),
                        workflow_id,
                        version: i64::from(version),
                        input,
                        parent_run_id: Some(claim.run_id.clone()),
                        grant_id: Some(required_str(&claim.policy_snapshot, "grantId")?.to_owned()),
                        test_mode: claim
                            .policy_snapshot
                            .get("testMode")
                            .and_then(Value::as_bool)
                            .unwrap_or(false),
                        breakpoint_node_ids: Vec::new(),
                    },
                    Some(&child_id),
                )
                .await?;
                batch_items::attach_child(pool, claim, &item_key, &child_id).await?;
                child_id
            };
            Ok::<_, String>((index, json!({"itemKey": item_key, "childRunId": child_id})))
        }
    }))
    .buffer_unordered(parallelism)
    .collect::<Vec<_>>()
    .await
    .into_iter()
    .collect::<Result<Vec<_>, _>>()?;
    children.sort_by_key(|(index, _)| *index);
    let children = children
        .into_iter()
        .map(|(_, child)| child)
        .collect::<Vec<_>>();
    waiting(
        WaitReason::Executor,
        json!({
            "childRuns": children,
            "completedItems": [],
            "errorPolicy": claim.resolved_input.get("errorPolicy")
                .and_then(Value::as_str).unwrap_or("stop"),
        }),
    )
}
