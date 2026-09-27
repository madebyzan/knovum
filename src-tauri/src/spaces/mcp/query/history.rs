use rmcp::ErrorData as McpError;
use tauri::AppHandle;

use super::super::support::{json_text, mcp_error};
use crate::spaces::mcp::arguments::HistoryArgs;
use crate::spaces::queries::knowledge_projection;

pub(in crate::spaces::mcp) async fn list_knowledge_events(
    app: &AppHandle,
    args: HistoryArgs,
) -> Result<String, McpError> {
    let value = knowledge_projection::list_knowledge_events(
        app,
        &args.space_id,
        args.entity_kind.as_deref(),
        args.entity_id.as_deref(),
        args.limit,
    )
    .await
    .map_err(mcp_error)?;
    json_text(value)
}
