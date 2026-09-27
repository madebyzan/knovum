use rmcp::ErrorData as McpError;
use tauri::AppHandle;

use super::super::support::{json_text, mcp_error};
use crate::spaces::queries::knowledge_projection;

pub(in crate::spaces::mcp) async fn list_spaces(
    app: &AppHandle,
    space_scope: Option<&str>,
) -> Result<String, McpError> {
    json_text(
        knowledge_projection::list_spaces(app, space_scope)
            .await
            .map_err(mcp_error)?,
    )
}
