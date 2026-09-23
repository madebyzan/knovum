use rmcp::ErrorData as McpError;
use tauri::AppHandle;

use super::super::support::{json_text, mcp_error};
use crate::spaces::queries::external;

pub(super) async fn run(app: &AppHandle, space_scope: Option<&str>) -> Result<String, McpError> {
    json_text(
        external::spaces(app, space_scope)
            .await
            .map_err(mcp_error)?,
    )
}
