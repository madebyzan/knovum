use rmcp::{handler::server::wrapper::Parameters, tool, tool_router, ErrorData as McpError};
use serde_json::json;

use super::{
    arguments::{
        PublishWorkflowArgs, SaveWorkflowDraftArgs, StartWorkflowArgs, ValidateWorkflowArgs,
        WorkflowSpaceArgs,
    },
    support::{json_text, mcp_error},
    SpaceMcpServer,
};
use crate::spaces::{actions, workflows};

#[tool_router(router = workflow_router, vis = "pub(super)")]
impl SpaceMcpServer {
    #[tool(description = "List editable drafts and immutable published Workflow versions")]
    async fn list_workflows(
        &self,
        Parameters(args): Parameters<WorkflowSpaceArgs>,
    ) -> Result<String, McpError> {
        self.require_space(&args.space_id)?;
        self.authorize_read(&args.space_id).await?;
        let drafts = workflows::list_workflow_drafts(self.app.clone(), args.space_id.clone())
            .await
            .map_err(mcp_error)?;
        let versions = workflows::list_workflow_versions(self.app.clone(), args.space_id)
            .await
            .map_err(mcp_error)?;
        json_text(json!({ "drafts": drafts, "versions": versions }))
    }

    #[tool(description = "Validate a Workflow definition without saving or executing it")]
    async fn validate_workflow(
        &self,
        Parameters(args): Parameters<ValidateWorkflowArgs>,
    ) -> Result<String, McpError> {
        match workflows::compile(&args.definition) {
            Ok(plan) => json_text(json!({ "valid": true, "plan": plan })),
            Err(diagnostics) => json_text(json!({ "valid": false, "diagnostics": diagnostics })),
        }
    }

    #[tool(description = "Save a Workflow draft through an exact user-approved Space Action")]
    async fn save_workflow_draft(
        &self,
        Parameters(args): Parameters<SaveWorkflowDraftArgs>,
    ) -> Result<String, McpError> {
        self.require_space(&args.space_id)?;
        let outcome = actions::invoke_with_app(
            &self.app,
            actions::ActionInvocationRequest {
                invocation_id: uuid::Uuid::new_v4().to_string(),
                idempotency_key: args.idempotency_key,
                grant_id: self.grant_id.clone(),
                space_id: args.space_id,
                action_id: "save-workflow-draft".into(),
                action_version: 1,
                parameters: json!({ "draft": args.draft }),
                input_snapshot: json!({}),
                expected_revisions: json!({}),
                approval_id: args.approval_id,
                parent_run_id: None,
            },
        )
        .await
        .map_err(mcp_error)?;
        json_text(outcome)
    }

    #[tool(description = "Publish an immutable Workflow version after exact approval")]
    async fn publish_workflow(
        &self,
        Parameters(args): Parameters<PublishWorkflowArgs>,
    ) -> Result<String, McpError> {
        self.require_space(&args.space_id)?;
        let outcome = actions::invoke_with_app(
            &self.app,
            actions::ActionInvocationRequest {
                invocation_id: uuid::Uuid::new_v4().to_string(),
                idempotency_key: args.idempotency_key,
                grant_id: self.grant_id.clone(),
                space_id: args.space_id,
                action_id: "publish-workflow".into(),
                action_version: 1,
                parameters: json!({
                    "workflowId": args.workflow_id,
                    "expectedRevision": args.expected_revision,
                }),
                input_snapshot: json!({}),
                expected_revisions: json!({"workflowDraft": args.expected_revision}),
                approval_id: args.approval_id,
                parent_run_id: None,
            },
        )
        .await
        .map_err(mcp_error)?;
        json_text(outcome)
    }

    #[tool(description = "Start one immutable published Workflow version with fixed input")]
    async fn start_workflow(
        &self,
        Parameters(args): Parameters<StartWorkflowArgs>,
    ) -> Result<String, McpError> {
        self.require_space(&args.space_id)?;
        let run = workflows::start_workflow(
            self.app.clone(),
            workflows::StartWorkflowRequest {
                space_id: args.space_id,
                workflow_id: args.workflow_id,
                version: args.version,
                input: args.input,
                parent_run_id: None,
                grant_id: Some(self.grant_id.clone()),
                test_mode: false,
                breakpoint_node_ids: Vec::new(),
            },
        )
        .await
        .map_err(mcp_error)?;
        json_text(run)
    }
}
