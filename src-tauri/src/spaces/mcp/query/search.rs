use rmcp::ErrorData as McpError;
use tauri::AppHandle;

use super::super::support::{json_text, mcp_error};
use crate::spaces::mcp::arguments::SearchArgs;
use crate::spaces::queries::knowledge_projection;

pub(in crate::spaces::mcp) async fn search_knowledge(
    app: &AppHandle,
    args: SearchArgs,
) -> Result<String, McpError> {
    let value =
        knowledge_projection::search_knowledge(app, &args.space_id, &args.query, args.limit)
            .await
            .map_err(mcp_error)?;
    json_text(value)
}
