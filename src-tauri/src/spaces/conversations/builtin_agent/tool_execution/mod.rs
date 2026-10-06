use serde_json::{json, Value};

pub(in crate::spaces::conversations) mod child;
mod media;
mod routing;

use crate::{
    agent_runtime::protocol::tools::{AgentToolInvocation, AgentToolOutcome},
    spaces::{
        actions::{self, ActionInvocationOutcome, ActionInvocationRequest},
        conversations::{
            execution::gateway::{tool_key, Gateway},
            tool_operations,
        },
        db,
    },
};

pub(in crate::spaces::conversations) async fn invoke(
    context: &Gateway,
    call: AgentToolInvocation,
) -> Result<Value, String> {
    context.require_tool(&call.name, &call.graph_namespace)?;
    super::identity::ensure_active(&context.pool, &context.ids.run_id).await?;
    if context
        .extensions
        .tools
        .iter()
        .any(|tool| tool.name == call.name)
    {
        return super::super::extensions::invoke_builtin(context, call).await;
    }
    let definition = context.catalog.action(&call.name)?;
    if call.checkpoint_id.is_empty() {
        return Err("builtin-agent-tool-intent-not-durable".into());
    }
    let invocation_id = tool_key(
        &call.identity,
        &call.graph_namespace,
        &call.message_id,
        &call.tool_call_id,
        &call.name,
    );
    let _intent = tool_operations::concurrency::acquire(&context.app, &invocation_id).await;
    let admitted: Option<i64> = sqlx::query_scalar(
        "SELECT 1 FROM spaces_builtin_agent_tool_admissions WHERE id=$1 AND task_id=$2",
    )
    .bind(&invocation_id)
    .bind(&context.ids.task_id)
    .fetch_optional(&context.pool)
    .await
    .map_err(db::sql_error)?;
    admitted.ok_or("builtin-agent-tool-not-admitted")?;
    let target_selection = routing::selection(context, definition, &invocation_id).await?;
    let operation = tool_operations::local_actions::start_tool_call(
        &context.pool,
        tool_operations::local_actions::LocalActionOperationStart {
            task_id: Some(&context.ids.task_id),
            run_id: &context.ids.run_id,
            graph_namespace: Some(&call.graph_namespace),
            checkpoint_id: Some(&call.checkpoint_id),
            message_id: Some(&call.message_id),
            provider_call_id: &call.tool_call_id,
            wire_name: &call.name,
            action_id: definition.id,
            invocation_id: &invocation_id,
            arguments: &call.arguments,
            target_selection,
        },
    )
    .await?;
    let value = if let Some(value) = operation.output {
        value
    } else {
        let mut parameters = call.arguments;
        let mut input_snapshot = parameters
            .as_object_mut()
            .and_then(|values| values.remove("inputNodeIds"))
            .map(|ids| json!({"inputNodeIds":ids}))
            .unwrap_or_else(|| json!({}));
        if let Some(target) = operation.target_selection {
            input_snapshot["targets"] = json!({definition.id: target});
        }
        let outcome = match actions::invoke(
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
        {
            Ok(outcome) => outcome,
            Err(error) => {
                let value =
                    json!({"isError":true,"content":[{"type":"text","text":error.to_string()}]});
                tool_operations::local_actions::complete_tool_call(
                    &context.pool,
                    &operation.id,
                    &value,
                )
                .await?;
                return serde_json::to_value(AgentToolOutcome::Completed {
                    value,
                    media: Vec::new(),
                })
                .map_err(db::json_error);
            }
        };
        let value = match outcome {
            ActionInvocationOutcome::AcceptedRun { run_id } => {
                match child::observe(&context.pool, &run_id).await? {
                    Some(value) => value,
                    None => {
                        return serde_json::to_value(AgentToolOutcome::AwaitChild { run_id })
                            .map_err(db::json_error)
                    }
                }
            }
            outcome => serde_json::to_value(outcome).map_err(db::json_error)?,
        };
        tool_operations::local_actions::complete_tool_call(&context.pool, &operation.id, &value)
            .await?;
        value
    };
    let media = media::read_node(&context.app, &context.target, definition.id, &value).await?;
    serde_json::to_value(AgentToolOutcome::Completed { value, media }).map_err(db::json_error)
}
