use rmcp::ErrorData as McpError;
use tauri::AppHandle;

use super::super::support::{json_text, mcp_error};
use crate::spaces::mcp::arguments::ProvenanceArgs;
use crate::spaces::queries::knowledge_projection;

pub(in crate::spaces::mcp) async fn list_provenance(
    app: &AppHandle,
    args: ProvenanceArgs,
) -> Result<String, McpError> {
    let value = knowledge_projection::list_provenance(
        app,
        &args.space_id,
        &args.target_kind,
        &args.target_id,
    )
    .await
    .map_err(mcp_error)?;
    json_text(value)
}
