use rmcp::{handler::server::wrapper::Parameters, tool, tool_router, ErrorData as McpError};
use serde_json::{json, Value};

use super::{
    arguments::{CommitKnowledgeArgs, InvokeActionArgs, SpaceArgs},
    support::{json_text, mcp_error},
    SpaceMcpServer,
};
use crate::spaces::actions;
mod invocation;

#[tool_router(router = action_router, vis = "pub(super)")]
impl SpaceMcpServer {
    #[tool(description = "List the authoritative Space Action catalog")]
    async fn list_space_actions(
        &self,
        Parameters(args): Parameters<SpaceArgs>,
    ) -> Result<String, McpError> {
        self.require_space(&args.space_id)?;
        self.authorize_read(&args.space_id).await?;
        // Desktop reasoning stays in the external runtime. These tools prepare/read/commit
        // knowledge and run deterministic processing; they never route inference back to it.
        json_text(
            actions::definitions()
                .iter()
                .filter(|action| is_external_agent_action(action))
                .collect::<Vec<_>>(),
        )
    }

    #[tool(description = "Invoke one registered Space Action through authorization and receipts")]
    async fn invoke_space_action(
        &self,
        Parameters(args): Parameters<InvokeActionArgs>,
    ) -> Result<String, McpError> {
        self.require_space(&args.space_id)?;
        require_platform_action(&args.action_id).map_err(mcp_error)?;
        let outcome = invocation::invoke(
            self,
            actions::ActionInvocationRequest {
                invocation_id: uuid::Uuid::new_v4().to_string(),
                idempotency_key: args.idempotency_key,
                grant_id: self.grant_id.clone(),
                space_id: args.space_id,
                action_id: args.action_id,
                action_version: args.action_version,
                parameters: Value::Object(args.parameters),
                input_snapshot: Value::Object(args.input_snapshot),
                expected_revisions: Value::Object(args.expected_revisions),
                parent_run_id: Some(self.parent_run_id.clone()),
            },
            "invoke_space_action",
        )
        .await
        .map_err(mcp_error)?;
        json_text(outcome)
    }

    #[tool(description = "Validate and atomically commit a Knowledge Change Set")]
    async fn commit_knowledge_changes(
        &self,
        Parameters(args): Parameters<CommitKnowledgeArgs>,
    ) -> Result<String, McpError> {
        self.require_space(&args.space_id)?;
        let outcome = invocation::invoke(
            self,
            actions::ActionInvocationRequest {
                invocation_id: uuid::Uuid::new_v4().to_string(),
                idempotency_key: args.idempotency_key,
                grant_id: self.grant_id.clone(),
                space_id: args.space_id,
                action_id: "commit-knowledge-changes".into(),
                action_version: 1,
                parameters: json!({"operations": args.operations}),
                input_snapshot: json!({}),
                expected_revisions: json!({}),
                parent_run_id: Some(self.parent_run_id.clone()),
            },
            "commit_knowledge_changes",
        )
        .await
        .map_err(mcp_error)?;
        json_text(outcome)
    }
}

fn require_platform_action(
    action_id: &str,
) -> Result<&'static actions::SpaceActionDefinition, String> {
    let definition = actions::require(action_id)?;
    if !is_external_agent_action(definition) {
        return Err("agent-inference-reentry: use your own runtime and publish the result; Knovum inference Actions are unavailable in desktop Agent sessions".into());
    }
    Ok(definition)
}

fn is_external_agent_action(action: &actions::SpaceActionDefinition) -> bool {
    action.routing_purpose.is_none()
}

pub(super) fn external_agent_action_ids() -> Vec<String> {
    actions::definitions()
        .iter()
        .filter(|action| is_external_agent_action(action))
        .map(|action| action.id.to_owned())
        .collect()
}

#[cfg(test)]
mod tests;
