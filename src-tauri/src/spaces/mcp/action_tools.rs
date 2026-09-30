use rmcp::{handler::server::wrapper::Parameters, tool, tool_router, ErrorData as McpError};
use serde_json::json;

use super::{
    arguments::{CommitKnowledgeArgs, InvokeActionArgs, SpaceArgs},
    support::{json_text, mcp_error},
    SpaceMcpServer,
};
use crate::spaces::actions;

#[tool_router(router = action_router, vis = "pub(super)")]
impl SpaceMcpServer {
    #[tool(description = "List the authoritative Space Action catalog")]
    async fn list_space_actions(
        &self,
        Parameters(args): Parameters<SpaceArgs>,
    ) -> Result<String, McpError> {
        self.require_space(&args.space_id)?;
        self.authorize_read(&args.space_id).await?;
        json_text(actions::definitions())
    }

    #[tool(description = "Invoke one registered Space Action through authorization and receipts")]
    async fn invoke_space_action(
        &self,
        Parameters(args): Parameters<InvokeActionArgs>,
    ) -> Result<String, McpError> {
        self.require_space(&args.space_id)?;
        let outcome = actions::invoke_with_app(
            &self.app,
            actions::ActionInvocationRequest {
                invocation_id: uuid::Uuid::new_v4().to_string(),
                idempotency_key: args.idempotency_key,
                grant_id: self.grant_id.clone(),
                space_id: args.space_id,
                action_id: args.action_id,
                action_version: args.action_version,
                parameters: args.parameters,
                input_snapshot: args.input_snapshot,
                expected_revisions: args.expected_revisions,
                parent_run_id: Some(self.parent_run_id.clone()),
            },
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
        let outcome = actions::invoke_with_app(
            &self.app,
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
        )
        .await
        .map_err(mcp_error)?;
        json_text(outcome)
    }
}
