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
            super::assistant::prompt_policy::SPACE_READING,
            super::assistant::prompt_policy::DESKTOP_TOOLS
        ))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn canvas_read_tools_are_discoverable_with_space_scoped_arguments() {
        let tools = super::SpaceMcpServer::tool_router().list_all();
        for name in [
            "list_space_contents",
            "search_knowledge",
            "get_node",
            "get_node_asset",
        ] {
            let tool = tools
                .iter()
                .find(|tool| tool.name == name)
                .expect("registered read tool");
            assert!(tool.input_schema["properties"].get("space_id").is_some());
            assert!(tool.annotations.as_ref().unwrap().read_only_hint.unwrap());
            if name == "get_node_asset" {
                assert!(tool.input_schema["required"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|field| field == "asset_revision"));
            }
        }
    }
}
