use rmcp::ErrorData as McpError;
use tauri::AppHandle;

use super::super::support::{json_text, mcp_error};
use crate::spaces::knowledge::grounding::{number_passages, Reading};
use crate::spaces::mcp::arguments::SearchArgs;
use crate::spaces::queries::knowledge_projection;

pub(in crate::spaces::mcp) async fn search_knowledge(
    app: &AppHandle,
    run_id: &str,
    args: SearchArgs,
) -> Result<String, McpError> {
    let pool = crate::spaces::db::pool(app).await.map_err(mcp_error)?;
    let mut tx = crate::spaces::db::begin_write(pool)
        .await
        .map_err(mcp_error)?;
    let mut value =
        knowledge_projection::search_knowledge(&mut tx, &args.space_id, &args.query, args.limit)
            .await
            .map_err(mcp_error)?;
    number_passages(&mut tx, run_id, Reading::Search, &mut value)
        .await
        .map_err(mcp_error)?;
    tx.commit()
        .await
        .map_err(|error| mcp_error(error.to_string()))?;
    json_text(value)
}
