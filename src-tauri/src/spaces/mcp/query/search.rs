use rmcp::ErrorData as McpError;
use tauri::AppHandle;

use super::super::support::{json_text, mcp_error};
use super::number_if_present;
use crate::spaces::knowledge::grounding::Reading;
use crate::spaces::mcp::arguments::SearchArgs;
use crate::spaces::queries::knowledge_projection::{self, SearchFilter, SearchOutputMode};

pub(in crate::spaces::mcp) async fn search_knowledge(
    app: &AppHandle,
    run_id: &str,
    args: SearchArgs,
) -> Result<String, McpError> {
    let mode = args.mode;
    let mut value = knowledge_projection::search_knowledge_hybrid(
        app,
        &args.space_id,
        &SearchFilter {
            query: args.query,
            limit: args.limit,
            node_ids: args.node_ids,
            roles: args.roles,
            mode,
        },
    )
    .await
    .map_err(mcp_error)?;
    if mode == SearchOutputMode::Passages {
        number_if_present(app, run_id, Reading::Search, &mut value).await?;
    }
    json_text(value)
}
