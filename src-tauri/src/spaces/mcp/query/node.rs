use rmcp::ErrorData as McpError;
use tauri::AppHandle;

use super::super::support::{json_text, mcp_error};
use crate::spaces::mcp::arguments::NodeArgs;
use crate::spaces::queries::knowledge_projection;

pub(in crate::spaces::mcp) async fn get_node(
    app: &AppHandle,
    args: NodeArgs,
) -> Result<String, McpError> {
    json_text(
        knowledge_projection::get_node(app, &args.space_id, &args.node_id)
            .await
            .map_err(mcp_error)?,
    )
}
