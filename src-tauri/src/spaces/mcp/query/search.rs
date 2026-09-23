use rmcp::ErrorData as McpError;
use tauri::AppHandle;

use super::super::support::{json_text, mcp_error};
use crate::spaces::mcp::arguments::SearchArgs;
use crate::spaces::queries::external;

pub(super) async fn run(app: &AppHandle, args: SearchArgs) -> Result<String, McpError> {
    let value = external::search(app, &args.space_id, &args.query, args.limit)
        .await
        .map_err(mcp_error)?;
    json_text(value)
}
