mod history;
mod node;
mod provenance;
mod search;
mod spaces;

use rmcp::ErrorData as McpError;
use tauri::AppHandle;

use super::arguments::{HistoryArgs, NodeArgs, ProvenanceArgs, SearchArgs};

pub(super) async fn get_history(app: &AppHandle, args: HistoryArgs) -> Result<String, McpError> {
    history::run(app, args).await
}

pub(super) async fn get_node(app: &AppHandle, args: NodeArgs) -> Result<String, McpError> {
    node::run(app, args).await
}

pub(super) async fn get_provenance(
    app: &AppHandle,
    args: ProvenanceArgs,
) -> Result<String, McpError> {
    provenance::run(app, args).await
}

pub(super) async fn search_knowledge(
    app: &AppHandle,
    args: SearchArgs,
) -> Result<String, McpError> {
    search::run(app, args).await
}

pub(super) async fn list_spaces(
    app: &AppHandle,
    space_scope: Option<&str>,
) -> Result<String, McpError> {
    spaces::run(app, space_scope).await
}
