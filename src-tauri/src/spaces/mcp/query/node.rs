use rmcp::ErrorData as McpError;
use tauri::AppHandle;

use super::super::support::{json_text, mcp_error};
use super::{acquire_reader, number_if_present};
use crate::spaces::knowledge::grounding::Reading;
use crate::spaces::mcp::arguments::NodeArgs;
use crate::spaces::queries::knowledge_projection;

pub(in crate::spaces::mcp) async fn get_node(
    app: &AppHandle,
    run_id: &str,
    args: NodeArgs,
) -> Result<String, McpError> {
    let mut reader = acquire_reader(app).await?;
    let mut value = knowledge_projection::load_node(&mut reader, &args.space_id, &args.node_id)
        .await
        .map_err(mcp_error)?;
    drop(reader);
    number_if_present(app, run_id, Reading::Node, &mut value).await?;
    json_text(value)
}
