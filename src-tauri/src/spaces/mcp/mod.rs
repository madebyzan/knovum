use tauri::AppHandle;

mod access;
mod action_tools;
pub(crate) mod agent_server;
mod arguments;
mod query;
mod read_tools;
mod relation_path;
mod run_tools;
mod support;
mod transport;
mod workflow_tools;

use super::authorization;
use support::mcp_error;

#[derive(Clone)]
struct SpaceMcpServer {
    app: AppHandle,
    space_scope: Option<String>,
    grant_id: String,
}

impl SpaceMcpServer {
    fn tool_router() -> rmcp::handler::server::router::tool::ToolRouter<Self> {
        Self::read_router() + Self::action_router() + Self::workflow_router() + Self::run_router()
    }
}

#[rmcp::tool_handler]
impl rmcp::ServerHandler for SpaceMcpServer {}
