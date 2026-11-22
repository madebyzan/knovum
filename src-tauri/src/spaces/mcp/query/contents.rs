use rmcp::ErrorData as McpError;
use tauri::AppHandle;

use super::super::support::{json_text, mcp_error};
use super::acquire_reader;
use crate::spaces::mcp::arguments::ContentsArgs;
use crate::spaces::queries::knowledge_projection;

pub(in crate::spaces::mcp) async fn list_space_contents(
    app: &AppHandle,
    args: ContentsArgs,
) -> Result<String, McpError> {
    let mut reader = acquire_reader(app).await?;
    let page =
        knowledge_projection::list_contents(&mut reader, &args.space_id, args.offset, args.limit)
            .await
            .map_err(mcp_error)?;
    json_text(page)
}
