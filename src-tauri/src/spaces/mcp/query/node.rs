use rmcp::ErrorData as McpError;
use tauri::AppHandle;

use super::super::support::{json_text, mcp_error};
use crate::spaces::mcp::arguments::NodeArgs;
use crate::spaces::queries::external;

pub(super) async fn run(app: &AppHandle, args: NodeArgs) -> Result<String, McpError> {
    json_text(
        external::node(app, &args.space_id, &args.node_id)
            .await
            .map_err(mcp_error)?,
    )
}
