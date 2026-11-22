use rmcp::ErrorData as McpError;
use tauri::AppHandle;

use super::super::support::{json_text, mcp_error};
use super::acquire_reader;
use crate::spaces::mcp::arguments::RelationsArgs;
use crate::spaces::queries::knowledge_projection;

pub(in crate::spaces::mcp) async fn list_object_types(
    app: &AppHandle,
    space_id: &str,
) -> Result<String, McpError> {
    let mut reader = acquire_reader(app).await?;
    let catalog = knowledge_projection::list_object_types(&mut reader, space_id)
        .await
        .map_err(mcp_error)?;
    json_text(catalog)
}

pub(in crate::spaces::mcp) async fn list_relations(
    app: &AppHandle,
    args: RelationsArgs,
) -> Result<String, McpError> {
    let mut reader = acquire_reader(app).await?;
    let relations = knowledge_projection::list_relations(
        &mut reader,
        &args.space_id,
        args.node_id.as_deref(),
        args.kind.as_deref(),
        args.limit,
    )
    .await
    .map_err(mcp_error)?;
    json_text(relations)
}
