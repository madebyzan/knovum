use serde_json::Value;

use crate::{
    agent_runtime::protocol::tools::{AgentToolInvocation, AgentToolOutcome},
    spaces::{
        conversations::{
            execution::gateway::{tool_key, Gateway},
            tool_operations::{self, ToolInvocation, ToolInvocationOutcome, ToolTarget},
        },
        db,
    },
};

/// Invokes a tool whose effect Knovum records as a conversation tool operation — an MCP tool or a
/// command — and hands the Agent its result, or tells it to wait for the user.
pub(super) async fn invoke(
    context: &Gateway,
    call: AgentToolInvocation,
    tool: ToolTarget<'_>,
) -> Result<Value, String> {
    if call.checkpoint_id.is_empty() {
        return Err("agent-tool-intent-not-durable".into());
    }
    let key = tool_key(
        &call.identity,
        &call.graph_namespace,
        &call.message_id,
        &call.tool_call_id,
        &call.name,
    );
    let admitted: Option<i64> = sqlx::query_scalar(
        "SELECT 1 FROM spaces_builtin_agent_tool_admissions WHERE id=$1 AND task_id=$2",
    )
    .bind(&key)
    .bind(&context.ids.task_id)
    .fetch_optional(&context.pool)
    .await
    .map_err(db::sql_error)?;
    admitted.ok_or("agent-tool-not-admitted")?;
    let outcome = tool_operations::invoke(
        &context.app,
        &context.pool,
        ToolInvocation {
            run_id: &context.ids.run_id,
            intent_id: &key,
            call_id: &call.tool_call_id,
            task_id: Some(&context.ids.task_id),
            graph_namespace: Some(&call.graph_namespace),
            checkpoint_id: Some(&call.checkpoint_id),
            message_id: Some(&call.message_id),
            tool,
            arguments: call.arguments,
        },
    )
    .await?;
    let outcome = match outcome {
        ToolInvocationOutcome::Completed(mut value) => {
            let media = tool_operations::media::project(&mut value, &context.target);
            AgentToolOutcome::Completed { value, media }
        }
        ToolInvocationOutcome::AwaitUser {
            operation_id,
            reason,
        } => AgentToolOutcome::AwaitUser {
            operation_id,
            reason: reason.into(),
        },
    };
    serde_json::to_value(outcome).map_err(db::json_error)
}
