use rmcp::ErrorData as McpError;

use super::{authorization, mcp_error, SpaceMcpServer};

impl SpaceMcpServer {
    pub(super) async fn authorize_read(&self, space_id: &str) -> Result<(), McpError> {
        authorization::require_read(&self.app, &self.turns.current()?.grant_id, space_id)
            .await
            .map_err(mcp_error)
    }

    pub(super) fn require_space(&self, requested: &str) -> Result<(), McpError> {
        if self
            .space_scope
            .as_deref()
            .is_none_or(|scope| scope == requested)
        {
            return Ok(());
        }
        Err(McpError::invalid_params(
            "This Agent session is scoped to a different Knovum Space",
            None,
        ))
    }

    pub(super) async fn authorize_run_control(
        &self,
        space_id: &str,
        run_id: &str,
    ) -> Result<(), McpError> {
        self.require_space(space_id)?;
        let turn = self.turns.current()?;
        authorization::require_run_control(&self.app, &turn.grant_id, space_id, run_id)
            .await
            .map_err(mcp_error)
    }
}
