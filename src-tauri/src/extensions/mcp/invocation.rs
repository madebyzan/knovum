use super::super::contracts::{AgentExtensionSnapshot, AgentExtensionTool, AgentToolBinding};
use super::{client::McpSession, McpInteraction};
use rmcp::{
    model::{
        CallToolRequestParams, GetPromptRequestParams, NumberOrString, ProgressToken,
        ReadResourceRequestParams, RequestMetaObject,
    },
    service::ServiceError,
};
use serde_json::{json, Value};
use std::{sync::Arc, time::Duration};
use tauri::AppHandle;

pub(crate) struct PreparedInvocation {
    session: Arc<McpSession>,
    binding: AgentToolBinding,
    arguments: Value,
    output_schema: Option<Value>,
}

pub(crate) enum InvocationObservation {
    Received(Value),
    OutcomeUnknown(&'static str),
}

pub(crate) async fn prepare(
    app: &AppHandle,
    snapshot: &AgentExtensionSnapshot,
    tool: &AgentExtensionTool,
    arguments: Value,
    interaction: Option<Arc<dyn McpInteraction>>,
) -> Result<PreparedInvocation, String> {
    let connection_id = match &tool.binding {
        AgentToolBinding::McpTool { connection_id, .. }
        | AgentToolBinding::McpReadResource { connection_id, .. }
        | AgentToolBinding::McpGetPrompt { connection_id, .. } => connection_id,
        AgentToolBinding::SpaceAction { .. } => return Err("mcp-invocation-source-mismatch".into()),
    };
    if !arguments.is_object() {
        return Err("mcp-tool-arguments-must-be-object".into());
    }
    let connection = snapshot
        .connections
        .iter()
        .find(|connection| &connection.id == connection_id)
        .ok_or("mcp-snapshot-connection-missing")?;
    jsonschema::validator_for(&tool.input_schema)
        .map_err(|error| error.to_string())?
        .validate(&arguments)
        .map_err(|error| format!("mcp-tool-arguments-invalid: {error}"))?;
    let output_schema = if let AgentToolBinding::McpTool {
        tool_name,
        schema_digest,
        ..
    } = &tool.binding
    {
        let definition = snapshot
            .catalogs
            .get(connection_id)
            .and_then(|catalog| {
                catalog
                    .tools
                    .iter()
                    .find(|definition| &definition.name == tool_name)
            })
            .ok_or("mcp-frozen-tool-missing")?;
        if &definition.schema_digest != schema_digest {
            return Err("mcp-tool-schema-identity-conflict".into());
        }
        definition.output_schema.clone()
    } else {
        None
    };
    let session = super::sessions::acquire(app, snapshot, connection, interaction).await?;
    super::super::snapshots::require_current(app, snapshot).await?;
    Ok(PreparedInvocation {
        session,
        binding: tool.binding.clone(),
        arguments,
        output_schema,
    })
}

impl PreparedInvocation {
    pub(crate) async fn execute(
        self,
        operation_id: &str,
        duration: Duration,
    ) -> InvocationObservation {
        let meta = RequestMetaObject::with_progress_token(ProgressToken(NumberOrString::String(
            operation_id.into(),
        )));
        let response = tokio::time::timeout(duration, async {
            match &self.binding {
                AgentToolBinding::McpTool { tool_name, .. } => {
                    let mut parameters = CallToolRequestParams::new(tool_name.clone())
                        .with_arguments(
                            self.arguments
                                .as_object()
                                .expect("validated object schema")
                                .clone(),
                        );
                    parameters.meta = Some(meta);
                    let response = self.session.service.call_tool(parameters).await?;
                    Ok(serde_json::to_value(response).expect("SDK response serializes"))
                }
                AgentToolBinding::McpReadResource { .. } => {
                    let params = ReadResourceRequestParams::new(
                        self.arguments["uri"]
                            .as_str()
                            .expect("validated resource URI"),
                    )
                    .with_meta(meta);
                    let response = self.session.service.read_resource(params).await?;
                    Ok(serde_json::to_value(response).expect("SDK response serializes"))
                }
                AgentToolBinding::McpGetPrompt { .. } => {
                    let mut params = GetPromptRequestParams::new(
                        self.arguments["name"]
                            .as_str()
                            .expect("validated prompt name"),
                    )
                    .with_meta(meta);
                    if let Some(arguments) = self.arguments.get("arguments") {
                        params = params.with_arguments(
                            arguments
                                .as_object()
                                .expect("validated prompt arguments")
                                .clone(),
                        );
                    }
                    let response = self.session.service.get_prompt(params).await?;
                    Ok(serde_json::to_value(response).expect("SDK response serializes"))
                }
                AgentToolBinding::SpaceAction { .. } => unreachable!("prepared MCP binding"),
            }
        })
        .await;
        let value = match response {
            Ok(Ok(value)) => value,
            Ok(Err(ServiceError::McpError(error))) => {
                return InvocationObservation::Received(
                    json!({"isError":true,"content":[{"type":"text","text":error.message}],"protocolError":error.code}),
                )
            }
            Ok(Err(_)) => return InvocationObservation::OutcomeUnknown("mcp-response-unavailable"),
            Err(_) => {
                self.session.cancel();
                return InvocationObservation::OutcomeUnknown("mcp-response-timeout");
            }
        };
        if serde_json::to_vec(&value)
            .expect("SDK response serializes")
            .len()
            > 8 * 1024 * 1024
        {
            return InvocationObservation::OutcomeUnknown("mcp-response-exceeds-8-mib");
        }
        if value["isError"] != true {
            if let Some(schema) = self.output_schema {
                let valid = jsonschema::validator_for(&schema).is_ok_and(|validator| {
                    value
                        .get("structuredContent")
                        .is_some_and(|content| validator.is_valid(content))
                });
                if !valid {
                    return InvocationObservation::Received(
                        json!({"isError":true,"content":[{"type":"text","text":"mcp-structured-output-schema-invalid"}],"received":value}),
                    );
                }
            }
        }
        InvocationObservation::Received(value)
    }
}
