use tauri::AppHandle;

mod access;
mod action_tools;
pub(crate) mod agent_server;
mod arguments;
mod node_asset;
mod query;
mod read_tools;
mod relation_path;
mod run_tools;
mod support;
mod transport;

use super::authorization;
use support::mcp_error;

#[derive(Clone)]
struct SpaceMcpServer {
    app: AppHandle,
    space_scope: Option<String>,
    grant_id: String,
    parent_run_id: String,
}

impl SpaceMcpServer {
    fn tool_router() -> rmcp::handler::server::router::tool::ToolRouter<Self> {
        Self::read_router() + Self::action_router() + Self::run_router()
    }
}

#[rmcp::tool_handler]
impl rmcp::ServerHandler for SpaceMcpServer {
    fn get_info(&self) -> rmcp::model::ServerInfo {
        let scope = self
            .space_scope
            .as_deref()
            .unwrap_or("Use list_spaces to select an authorized Space");
        rmcp::model::ServerInfo::new(
            rmcp::model::ServerCapabilities::builder()
                .enable_tools()
                .build(),
        )
        .with_server_info(rmcp::model::Implementation::new(
            "knovum",
            env!("CARGO_PKG_VERSION"),
        ))
        .with_instructions(format!(
            "Current Space: {scope}. {} {}",
            super::conversations::prompt_policy::SPACE_READING,
            super::conversations::prompt_policy::DESKTOP_TOOLS
        ))
    }
}

#[cfg(test)]
mod tests;
