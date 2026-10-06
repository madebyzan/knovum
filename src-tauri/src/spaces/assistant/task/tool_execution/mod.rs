use serde_json::{json, Value};

pub(in crate::spaces::assistant) mod child;
mod media;
mod routing;

use crate::{
    agent_runtime::protocol::tools::{AssistantToolInvocation, AssistantToolOutcome},
    spaces::{
        actions::{self, ActionInvocationOutcome, ActionInvocationRequest},
        assistant::{
            execution::gateway::{tool_key, Gateway},
            repository,
        },
        db,
    },
};

pub(in crate::spaces::assistant) async fn invoke(
    context: &Gateway,
    call: AssistantToolInvocation,
) -> Result<Value, String> {
    context.require_tool(&call.name, &call.graph_namespace)?;
    super::identity::ensure_active(&context.pool, &context.ids.run_id).await?;
    let definition = context
        .catalog
        .by_wire_name
        .get(&call.name)
        .ok_or("assistant-action-not-allowed")?;
    if call.checkpoint_id.is_empty() {
        return Err("assistant-tool-intent-not-durable".into());
    }
    let invocation_id = tool_key(
        &call.identity,
        &call.graph_namespace,
        &call.message_id,
        &call.tool_call_id,
        &call.name,
    );
    let admitted: Option<i64> = sqlx::query_scalar(
        "SELECT 1 FROM spaces_assistant_tool_admissions WHERE id=$1 AND task_id=$2",
    )
    .bind(&invocation_id)
    .bind(&context.ids.task_id)
    .fetch_optional(&context.pool)
    .await
    .map_err(db::sql_error)?;
    admitted.ok_or("assistant-tool-not-admitted")?;
    let target_selection = routing::selection(context, definition, &invocation_id).await?;
    let operation = repository::start_tool_call(
        &context.pool,
        repository::AssistantToolCallStart {
            task_id: &context.ids.task_id,
            run_id: &context.ids.run_id,
            graph_namespace: &call.graph_namespace,
            message_id: &call.message_id,
            provider_call_id: &call.tool_call_id,
            wire_name: &call.name,
            action_id: definition.id,
            invocation_id: &invocation_id,
            arguments: &call.arguments,
            target_selection,
        },
    )
    .await?;
    let mut parameters = call.arguments;
    let mut input_snapshot = parameters
        .as_object_mut()
        .and_then(|values| values.remove("inputNodeIds"))
        .map(|ids| json!({"inputNodeIds":ids}))
        .unwrap_or_else(|| json!({}));
    if let Some(target) = operation.target_selection {
        input_snapshot["targets"] = json!({definition.id: target});
    }
    let outcome = actions::invoke(
        &context.pool,
        ActionInvocationRequest {
            invocation_id: invocation_id.clone(),
            idempotency_key: invocation_id,
            grant_id: context.ids.grant_id.clone(),
            space_id: context.request.space_id.clone(),
            action_id: definition.id.into(),
            action_version: definition.version,
            parameters,
            input_snapshot,
            expected_revisions: Value::Null,
            parent_run_id: Some(context.ids.run_id.clone()),
        },
    )
    .await
    .map_err(|error| error.to_string())?;
    let value = if let Some(value) = operation.output {
        value
    } else {
        let value = match outcome {
            ActionInvocationOutcome::AcceptedRun { run_id } => {
                match child::observe(&context.pool, &run_id).await? {
                    Some(value) => value,
                    None => {
                        return serde_json::to_value(AssistantToolOutcome::AwaitChild { run_id })
                            .map_err(db::json_error)
                    }
                }
            }
            outcome => serde_json::to_value(outcome).map_err(db::json_error)?,
        };
        repository::complete_tool_call(
            &context.pool,
            repository::AssistantToolCallCompletion {
                id: &operation.id,
                run_id: &context.ids.run_id,
                conversation_id: &context.ids.conversation_id,
                branch_id: &context.ids.branch_id,
                call_id: &call.tool_call_id,
                name: definition.id,
                tool_output: &value,
            },
        )
        .await?;
        value
    };
    let media = media::read_node(&context.app, &context.target, definition.id, &value).await?;
    serde_json::to_value(AssistantToolOutcome::Completed { value, media }).map_err(db::json_error)
}
