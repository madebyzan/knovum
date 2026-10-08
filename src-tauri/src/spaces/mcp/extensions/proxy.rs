use super::super::{support::mcp_error, SpaceMcpServer};
use crate::spaces::{
    conversations::tool_operations::{self, ToolInvocation, ToolInvocationOutcome},
    db, runs,
};
use rmcp::{
    model::{CallToolRequestParams, CallToolResponse, CallToolResult},
    service::RequestContext,
    ErrorData, RoleServer,
};

pub(super) async fn invoke(
    server: &SpaceMcpServer,
    request: CallToolRequestParams,
    context: RequestContext<RoleServer>,
) -> Result<CallToolResponse, ErrorData> {
    let name = request.name.into_owned();
    let arguments = serde_json::Value::Object(request.arguments.unwrap_or_default());
    let value = invoke_value(server, &name, arguments, context).await?;
    let tool_output = if value.get("content").is_some() {
        serde_json::from_value::<CallToolResult>(value)
            .map_err(|error| mcp_error(error.to_string()))?
    } else {
        CallToolResult::success(vec![rmcp::model::ContentBlock::text(
            serde_json::to_string(&value).map_err(|error| mcp_error(error.to_string()))?,
        )])
    };
    Ok(CallToolResponse::Complete(tool_output))
}

pub(super) async fn invoke_value(
    server: &SpaceMcpServer,
    name: &str,
    arguments: serde_json::Value,
    context: RequestContext<RoleServer>,
) -> Result<serde_json::Value, ErrorData> {
    let snapshot = server
        .conversation_extensions
        .as_ref()
        .ok_or_else(|| mcp_error("Agent extensions not selected"))?;
    let tool = snapshot
        .tools
        .iter()
        .find(|tool| tool.name == name)
        .ok_or_else(|| mcp_error("MCP tool not selected"))?;
    let pool = db::pool(&server.app).await.map_err(mcp_error)?;
    let (session_id, turn_id): (String,String) = sqlx::query_as("SELECT c.id,t.id FROM spaces_external_agent_conversations c JOIN spaces_external_agent_turns t ON t.conversation_id=c.id WHERE c.run_id=$1 AND t.state='running' ORDER BY t.created_at DESC LIMIT 1")
        .bind(&server.parent_run_id).fetch_optional(pool).await.map_err(|error| mcp_error(error.to_string()))?.ok_or_else(|| mcp_error("external Agent turn is not active"))?;
    let call_id =
        serde_json::to_string(&context.id).map_err(|error| mcp_error(error.to_string()))?;
    let intent = format!("acp:{session_id}:{turn_id}:{call_id}");
    loop {
        let outcome = tool_operations::invoke(
            &server.app,
            pool,
            snapshot,
            ToolInvocation {
                run_id: &server.parent_run_id,
                intent_id: &intent,
                call_id: &call_id,
                task_id: None,
                graph_namespace: None,
                checkpoint_id: None,
                message_id: None,
                tool,
                arguments: arguments.clone(),
            },
        )
        .await
        .map_err(mcp_error)?;
        match outcome {
            ToolInvocationOutcome::Completed(value) => return Ok(value),
            ToolInvocationOutcome::AwaitUser {
                operation_id,
                reason,
            } => {
                let waiting = sqlx::query("UPDATE spaces_runs SET state='waiting',wait_reason=$1,state_version=state_version+1 WHERE id=$2 AND completed_at IS NULL AND cancel_requested_at IS NULL AND pause_requested_at IS NULL").bind(reason).bind(&server.parent_run_id).execute(pool).await.map_err(|error| mcp_error(error.to_string()))?;
                db::require_one(waiting, "conversation-tool-run-inactive").map_err(mcp_error)?;
                runs::announce_change();
                tokio::select! {
                    tool_output = tool_operations::intervention::wait(pool, &server.parent_run_id, &operation_id) => tool_output.map_err(mcp_error)?,
                    _ = context.ct.cancelled() => return Err(mcp_error("external Agent request cancelled")),
                }
                let resumed = sqlx::query("UPDATE spaces_runs SET state='running',wait_reason=NULL,state_version=state_version+1 WHERE id=$1 AND state IN ('waiting','running') AND completed_at IS NULL AND cancel_requested_at IS NULL AND pause_requested_at IS NULL").bind(&server.parent_run_id).execute(pool).await.map_err(|error| mcp_error(error.to_string()))?;
                db::require_one(resumed, "conversation-tool-run-inactive").map_err(mcp_error)?;
            }
        }
    }
}
