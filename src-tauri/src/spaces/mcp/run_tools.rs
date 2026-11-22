use rmcp::{handler::server::wrapper::Parameters, tool, tool_router, ErrorData as McpError};
use serde_json::json;

use super::{
    arguments::{ControlRunArgs, GetRunArgs, RunControlCommand},
    support::{json_text, mcp_error},
    SpaceMcpServer,
};
use crate::spaces::{run_control, runs};

#[tool_router(router = run_router, vis = "pub(super)")]
impl SpaceMcpServer {
    #[tool(
        description = "Read a durable Run, its Step states, and optionally its event log after after_sequence",
        annotations(read_only_hint = true)
    )]
    async fn get_run(&self, Parameters(args): Parameters<GetRunArgs>) -> Result<String, McpError> {
        let run = self.authorized_run(&args.space_id, &args.run_id).await?;
        let steps = runs::list_run_steps(self.app.clone(), args.run_id.clone())
            .await
            .map_err(mcp_error)?;
        let mut payload = json!({ "run": run, "steps": steps });
        if args.include_events {
            let events = runs::list_run_events(self.app.clone(), args.run_id, args.after_sequence)
                .await
                .map_err(mcp_error)?;
            payload["events"] =
                serde_json::to_value(events).map_err(|error| mcp_error(error.to_string()))?;
        }
        json_text(payload)
    }

    #[tool(
        description = "Control a durable Run: pause, resume, cancel, or selectively retry failed step_ids"
    )]
    async fn control_run(
        &self,
        Parameters(args): Parameters<ControlRunArgs>,
    ) -> Result<String, McpError> {
        self.authorize_run_control(&args.space_id, &args.run_id)
            .await?;
        match args.command {
            RunControlCommand::Pause => {
                let receipt = run_control::pause_run(self.app.clone(), args.run_id)
                    .await
                    .map_err(mcp_error)?;
                json_text(json!({
                    "accepted": true,
                    "interruptionErrors": receipt.interruption_errors,
                }))
            }
            RunControlCommand::Resume => {
                run_control::resume_run(self.app.clone(), args.run_id)
                    .await
                    .map_err(mcp_error)?;
                json_text(json!({ "accepted": true }))
            }
            RunControlCommand::Cancel => {
                let receipt = run_control::cancel_run(self.app.clone(), args.run_id)
                    .await
                    .map_err(mcp_error)?;
                json_text(json!({
                    "accepted": true,
                    "interruptionErrors": receipt.interruption_errors,
                }))
            }
            RunControlCommand::Retry => {
                let retry = runs::retry_run(self.app.clone(), args.run_id, args.step_ids)
                    .await
                    .map_err(mcp_error)?;
                json_text(retry)
            }
        }
    }
}

impl SpaceMcpServer {
    async fn authorized_run(
        &self,
        space_id: &str,
        run_id: &str,
    ) -> Result<runs::RunObservation, McpError> {
        self.require_space(space_id)?;
        self.authorize_read(space_id).await?;
        let run = runs::get_run(self.app.clone(), run_id.to_owned())
            .await
            .map_err(mcp_error)?;
        if run.scope.require_space_id().map_err(mcp_error)? != space_id {
            return Err(McpError::invalid_params(
                "Run belongs to another Space",
                None,
            ));
        }
        Ok(run)
    }
}
