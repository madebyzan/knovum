use rmcp::ErrorData as McpError;
use tauri::AppHandle;

use super::super::support::{json_text, mcp_error};
use crate::spaces::mcp::arguments::HistoryArgs;
use crate::spaces::queries::external;

pub(super) async fn run(app: &AppHandle, args: HistoryArgs) -> Result<String, McpError> {
    let value = external::history(
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
