use rmcp::ErrorData as McpError;
use tauri::AppHandle;

use super::super::support::{json_text, mcp_error};
use crate::spaces::mcp::arguments::NodeArgs;
use crate::spaces::queries::knowledge_projection;

pub(in crate::spaces::mcp) async fn get_node(
    app: &AppHandle,
    args: NodeArgs,
) -> Result<String, McpError> {
    let pool = crate::spaces::db::pool(app).await.map_err(mcp_error)?;
    let mut tx = pool
        .begin()
        .await
        .map_err(|error| mcp_error(error.to_string()))?;
    json_text(
        knowledge_projection::load_node(&mut tx, &args.space_id, &args.node_id)
            .await
            .map_err(mcp_error)?,
    )
}
