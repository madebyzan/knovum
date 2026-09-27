use rmcp::{handler::server::wrapper::Parameters, tool, tool_router, ErrorData as McpError};
use serde_json::json;

use super::{
    arguments::{RetryRunArgs, RunArgs, RunEventsArgs},
    support::{json_text, mcp_error},
    SpaceMcpServer,
};
use crate::spaces::runs;

#[tool_router(router = run_router, vis = "pub(super)")]
impl SpaceMcpServer {
    #[tool(description = "Read a durable Run and all current Step states")]
    async fn get_run(&self, Parameters(args): Parameters<RunArgs>) -> Result<String, McpError> {
        let run = self.authorized_run(&args).await?;
        let steps = runs::list_run_steps(self.app.clone(), args.run_id.clone())
            .await
            .map_err(mcp_error)?;
        json_text(json!({ "run": run, "steps": steps }))
    }

    #[tool(description = "Read durable Run events after a sequence cursor")]
    async fn list_run_events(
        &self,
        Parameters(args): Parameters<RunEventsArgs>,
    ) -> Result<String, McpError> {
        self.authorized_run(&RunArgs {
            space_id: args.space_id,
            run_id: args.run_id.clone(),
        })
        .await?;
        let events = runs::list_run_events(self.app.clone(), args.run_id, args.after_sequence)
            .await
            .map_err(mcp_error)?;
        json_text(events)
    }

    #[tool(description = "Pause a durable Run at the next safe boundary")]
    async fn pause_run(&self, Parameters(args): Parameters<RunArgs>) -> Result<String, McpError> {
        self.authorize_run_control(&args.space_id, &args.run_id)
            .await?;
        runs::pause_run(self.app.clone(), args.run_id)
            .await
            .map_err(mcp_error)?;
        json_text(json!({"accepted": true}))
    }

    #[tool(description = "Resume a paused durable Run")]
    async fn resume_run(&self, Parameters(args): Parameters<RunArgs>) -> Result<String, McpError> {
        self.authorize_run_control(&args.space_id, &args.run_id)
            .await?;
        runs::resume_run(self.app.clone(), args.run_id)
            .await
            .map_err(mcp_error)?;
        json_text(json!({"accepted": true}))
    }

    #[tool(description = "Cancel a durable Run and stop future publication")]
    async fn cancel_run(&self, Parameters(args): Parameters<RunArgs>) -> Result<String, McpError> {
        self.authorize_run_control(&args.space_id, &args.run_id)
            .await?;
        runs::cancel_run(self.app.clone(), args.run_id)
            .await
            .map_err(mcp_error)?;
        json_text(json!({"accepted": true}))
    }

    #[tool(description = "Create a selective retry Run for failed or affected Steps")]
    async fn retry_run(
        &self,
        Parameters(args): Parameters<RetryRunArgs>,
    ) -> Result<String, McpError> {
        self.authorize_run_control(&args.space_id, &args.run_id)
            .await?;
        let retry = runs::retry_run(self.app.clone(), args.run_id, args.step_ids)
            .await
            .map_err(mcp_error)?;
        json_text(retry)
    }
}

impl SpaceMcpServer {
    async fn authorized_run(&self, args: &RunArgs) -> Result<runs::RunObservation, McpError> {
        self.require_space(&args.space_id)?;
        self.authorize_read(&args.space_id).await?;
        let run = runs::get_run(self.app.clone(), args.run_id.clone())
            .await
            .map_err(mcp_error)?;
        if run.home_space_id != args.space_id {
            return Err(McpError::invalid_params(
                "Run belongs to another Space",
                None,
            ));
        }
        Ok(run)
    }
}
