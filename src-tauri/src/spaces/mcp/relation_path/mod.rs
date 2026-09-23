use rmcp::ErrorData as McpError;
use serde_json::Value;
use tauri::AppHandle;

use super::mcp_error;
use crate::spaces::queries::external;

pub(super) async fn find(
    app: &AppHandle,
    space_id: &str,
    source_id: &str,
    target_id: &str,
    max_depth: usize,
) -> Result<Vec<Value>, McpError> {
    external::relation_path(app, space_id, source_id, target_id, max_depth)
        .await
        .map_err(mcp_error)
}

#[cfg(test)]
mod tests;
