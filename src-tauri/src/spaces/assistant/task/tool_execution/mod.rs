use crate::app_error::AppError;
use serde_json::{json, Value};
use sqlx::SqlitePool;

use super::identity::{ensure_active, TaskIds};
mod media;
mod settlement;
use crate::{
    agent_runtime::{RuntimeToolCall, RuntimeToolReply},
    spaces::{
        actions::{self, ActionInvocationRequest},
        assistant::repository,
    },
};

pub(super) async fn execute(
    context: &super::driver::DriverContext<'_>,
    turn_id: &str,
    calls: Vec<RuntimeToolCall>,
) -> Result<Vec<RuntimeToolReply>, AppError> {
    let super::driver::DriverContext {
        app,
        pool,
        target,
        request,
        ids,
        catalog,
        ..
    } = context;
    let mut results = Vec::with_capacity(calls.len());
    for call in calls {
        ensure_active(pool, &ids.run_id).await?;
        let definition = catalog
            .by_wire_name
            .get(&call.name)
            .ok_or("tool-not-allowed: model requested an unavailable tool")?;
        let action_id = definition.id.to_owned();
        let invocation_id = format!("{}:{}", ids.task_id, call.call_id);
        let record = repository::start_tool_call(
            pool,
            repository::AssistantToolCallStart {
                task_id: &ids.task_id,
                run_id: &ids.run_id,
                model_turn_id: turn_id,
                provider_call_id: &call.call_id,
                wire_name: &call.name,
                action_id: &action_id,
                invocation_id: &invocation_id,
                arguments: &call.arguments,
            },
        )
        .await?;
        let value = if let Some(value) = record.result {
            value
        } else {
            let mut parameters = call.arguments.clone();
            let mut input_snapshot = parameters
                .as_object_mut()
                .and_then(|values| values.remove("inputNodeIds"))
                .map(|node_ids| json!({"inputNodeIds": node_ids}))
                .unwrap_or_else(|| json!({}));
            if definition.routing_purpose.is_some() {
                input_snapshot["targets"] = json!({});
                input_snapshot["targets"][definition.id] = json!(&request.target);
            }
            let invocation = ActionInvocationRequest {
                invocation_id: invocation_id.clone(),
                idempotency_key: invocation_id,
                grant_id: ids.grant_id.clone(),
                space_id: request.space_id.clone(),
                action_id: definition.id.into(),
                action_version: definition.version,
                parameters,
                input_snapshot,
                expected_revisions: Value::Null,
                parent_run_id: Some(ids.run_id.clone()),
            };
            let outcome = actions::invoke(pool, invocation).await?;
            let value = settlement::settle(pool, &ids.run_id, outcome).await?;
            complete_tool(pool, ids, &call, &record.id, &action_id, value.clone()).await?;
            value
        };
        let media = media::read_node(app, target, &request.target, &action_id, &value).await?;
        results.push(RuntimeToolReply {
            call_id: call.call_id,
            name: call.name,
            result: value,
            media,
        });
    }
    Ok(results)
}

async fn complete_tool(
    pool: &SqlitePool,
    ids: &TaskIds,
    call: &RuntimeToolCall,
    row_id: &str,
    action_id: &str,
    value: Value,
) -> Result<(), AppError> {
    Ok(repository::complete_tool_call(
        pool,
        repository::AssistantToolCallCompletion {
            id: row_id,
            run_id: &ids.run_id,
            conversation_id: &ids.conversation_id,
            branch_id: &ids.branch_id,
            call_id: &call.call_id,
            name: action_id,
            tool_output: &value,
        },
    )
    .await?)
}
