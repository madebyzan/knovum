use rmcp::ErrorData as McpError;
use tauri::AppHandle;

use super::super::support::{json_text, mcp_error};
use crate::spaces::mcp::arguments::SearchArgs;
use crate::spaces::queries::knowledge_projection;

pub(in crate::spaces::mcp) async fn search_knowledge(
    app: &AppHandle,
    args: SearchArgs,
) -> Result<String, McpError> {
    let pool = crate::spaces::db::pool(app).await.map_err(mcp_error)?;
    let mut connection = pool
        .acquire()
        .await
        .map_err(|error| mcp_error(error.to_string()))?;
    let value = knowledge_projection::search_knowledge(
        &mut connection,
        &args.space_id,
        &args.query,
        args.limit,
    )
    .await
    .map_err(mcp_error)?;
    json_text(value)
}
