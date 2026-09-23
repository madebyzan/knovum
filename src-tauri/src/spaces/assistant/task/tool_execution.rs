use serde_json::{json, Value};
use sqlx::SqlitePool;

use super::identity::{ensure_active, TaskIds};
mod settlement;
use crate::{
    agent_runtime::{RuntimeToolCall, RuntimeToolResult},
    spaces::{
        actions::{self, ActionInvocationRequest},
        assistant::{repository, tools, StartAssistantTaskRequest},
    },
};

pub(super) async fn execute(
    pool: &SqlitePool,
    request: &StartAssistantTaskRequest,
    ids: &TaskIds,
    catalog: &tools::ToolCatalog,
    turn_id: &str,
    calls: Vec<RuntimeToolCall>,
) -> Result<Vec<RuntimeToolResult>, String> {
    let mut results = Vec::with_capacity(calls.len());
    for call in calls {
        ensure_active(pool, &ids.run_id).await?;
        let binding = catalog
            .by_wire_name
            .get(&call.name)
            .ok_or("tool-not-allowed: model requested an unavailable tool")?;
        let action_id = match binding {
            tools::ToolBinding::Action(definition) => definition.id.to_owned(),
            tools::ToolBinding::External(tool) => format!("external-tool:{}", tool.id),
        };
        let invocation_id = format!("{}:{}", ids.task_id, call.call_id);
        let record = repository::start_tool_call(
            pool,
            &ids.task_id,
            turn_id,
            &call.call_id,
            &call.name,
            &action_id,
            &invocation_id,
        )
        .await?;
        if let Some(value) = record.result {
            results.push(RuntimeToolResult {
                call_id: call.call_id,
                name: call.name,
                result: value,
            });
            continue;
        }
        let mut parameters = call.arguments.clone();
        if let tools::ToolBinding::External(tool) = binding {
            if record.resumed {
                return Err(
                    "external-tool-outcome-unknown: reconcile the previous call before continuing"
                        .into(),
                );
            }
            super::external_tool::approve_if_needed(pool, ids, tool, &parameters).await?;
            let value = crate::spaces::workflows::external_tools::execute_direct(
                pool,
                &request.space_id,
                &ids.grant_id,
                &tool.id,
                &tool.schema_fingerprint,
                &parameters,
            )
            .await?;
            complete_tool(pool, ids, &call, &record.id, value.clone()).await?;
            results.push(RuntimeToolResult {
                call_id: call.call_id,
                name: call.name,
                result: value,
            });
            continue;
        }
        let tools::ToolBinding::Action(definition) = binding else {
            unreachable!()
        };
        let input_snapshot = parameters
            .as_object_mut()
            .and_then(|values| values.remove("inputNodeIds"))
            .map(|node_ids| json!({"inputNodeIds": node_ids}))
            .unwrap_or(Value::Null);
        let mut invocation = ActionInvocationRequest {
            invocation_id: invocation_id.clone(),
            idempotency_key: invocation_id,
            grant_id: ids.grant_id.clone(),
            space_id: request.space_id.clone(),
            action_id: definition.id.into(),
            action_version: definition.version,
            parameters,
            input_snapshot,
            expected_revisions: Value::Null,
            approval_id: None,
            parent_run_id: Some(ids.run_id.clone()),
        };
        let outcome = actions::invoke(pool, invocation.clone()).await?;
        let value = settlement::settle(pool, &ids.run_id, &mut invocation, outcome).await?;
        complete_tool(pool, ids, &call, &record.id, value.clone()).await?;
        results.push(RuntimeToolResult {
            call_id: call.call_id,
            name: call.name,
            result: value,
        });
    }
    Ok(results)
}

async fn complete_tool(
    pool: &SqlitePool,
    ids: &TaskIds,
    call: &RuntimeToolCall,
    row_id: &str,
    value: Value,
) -> Result<(), String> {
    repository::complete_tool_call(
        pool,
        row_id,
        &ids.conversation_id,
        &ids.branch_id,
        &call.call_id,
        &call.name,
        &value,
    )
    .await
}
