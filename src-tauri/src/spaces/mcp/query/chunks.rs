use rmcp::ErrorData as McpError;
use tauri::AppHandle;

use super::super::support::{json_text, mcp_error};
use super::{acquire_reader, number_if_present};
use crate::spaces::knowledge::grounding::Reading;
use crate::spaces::mcp::arguments::NodeChunksArgs;
use crate::spaces::queries::knowledge_projection::{self, ChunkReadFilter};

pub(in crate::spaces::mcp) async fn read_node_chunks(
    app: &AppHandle,
    run_id: &str,
    args: NodeChunksArgs,
) -> Result<String, McpError> {
    let mut reader = acquire_reader(app).await?;
    let mut value = knowledge_projection::load_node_chunks(
        &mut reader,
        &args.space_id,
        &args.node_id,
        ChunkReadFilter {
            role: args.role,
            offset: args.offset,
            limit: args.limit,
            around_chunk_id: args.around_chunk_id,
            context_chunks: args.context_chunks,
        },
    )
    .await
    .map_err(mcp_error)?;
    drop(reader);
    number_if_present(app, run_id, Reading::Node, &mut value).await?;
    json_text(value)
}
