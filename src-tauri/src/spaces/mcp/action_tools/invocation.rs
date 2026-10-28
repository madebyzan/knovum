use super::super::SpaceMcpServer;
use crate::spaces::{actions, conversations::tool_operations, db};
use serde_json::{json, Value};

/// Runs a Space Action an Agent asked for during its turn, recorded under the turn's Run.
pub(super) async fn invoke(
    server: &SpaceMcpServer,
    run_id: &str,
    mut request: actions::ActionInvocationRequest,
    wire_name: &str,
) -> Result<Value, String> {
    let pool = db::pool(&server.app).await?;
    let intent = format!("acp-action:{run_id}:{}", request.idempotency_key);
    let _intent = tool_operations::concurrency::acquire(&server.app, &intent).await;
    let arguments = json!({"actionId":request.action_id,"actionVersion":request.action_version,"parameters":request.parameters,"inputSnapshot":request.input_snapshot,"expectedRevisions":request.expected_revisions});
    let operation = tool_operations::local_actions::start_tool_call(
        pool,
        tool_operations::local_actions::LocalActionOperationStart {
            task_id: None,
            run_id,
            graph_namespace: None,
            checkpoint_id: None,
            message_id: None,
            provider_call_id: &request.idempotency_key,
            wire_name,
            action_id: &request.action_id,
            invocation_id: &intent,
            arguments: &arguments,
            target_selection: None,
        },
    )
    .await?;
    if let Some(receipt) = operation.output {
        return Ok(receipt);
    }
    request.invocation_id = intent;
    let receipt = match actions::invoke_with_app(&server.app, request).await {
        Ok(outcome) => serde_json::to_value(outcome).map_err(db::json_error)?,
        Err(error) => json!({"isError":true,"content":[{"type":"text","text":error.to_string()}]}),
    };
    tool_operations::local_actions::complete_tool_call(pool, &operation.id, &receipt).await?;
    Ok(receipt)
}
