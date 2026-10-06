use tauri::AppHandle;

mod access;
mod action_tools;
pub(crate) mod agent_server;
mod arguments;
mod extensions;
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
    conversation_extensions:
        Option<std::sync::Arc<crate::extensions::contracts::AgentExtensionSnapshot>>,
}

impl SpaceMcpServer {
    fn tool_router() -> rmcp::handler::server::router::tool::ToolRouter<Self> {
        Self::read_router()
            + Self::action_router()
            + Self::run_router()
            + Self::extension_skill_router()
    }
}

#[rmcp::tool_handler]
impl rmcp::ServerHandler for SpaceMcpServer {
    async fn call_tool(
        &self,
        request: rmcp::model::CallToolRequestParams,
        context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<rmcp::model::CallToolResponse, rmcp::ErrorData> {
        self.dispatch_tool(request, context).await
    }
    async fn list_tools(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<rmcp::model::ListToolsResult, rmcp::ErrorData> {
        Ok(self.tool_listing())
    }
    fn get_tool(&self, name: &str) -> Option<rmcp::model::Tool> {
        Self::tool_router().get(name).cloned().or_else(|| {
            self.extension_tools()
                .into_iter()
                .find(|tool| tool.name == name)
        })
    }

    async fn list_resources(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<rmcp::model::ListResourcesResult, rmcp::ErrorData> {
        Ok(rmcp::model::ListResourcesResult::with_all_items(
            extensions::catalog_projection::selected_resources(
                self.conversation_extensions.as_deref(),
            )
            .into_iter()
            .map(|binding| binding.declaration)
            .collect(),
        ))
    }
    async fn list_resource_templates(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<rmcp::model::ListResourceTemplatesResult, rmcp::ErrorData> {
        Ok(rmcp::model::ListResourceTemplatesResult::with_all_items(
            Vec::new(),
        ))
    }
    async fn read_resource(
        &self,
        request: rmcp::model::ReadResourceRequestParams,
        context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<rmcp::model::ReadResourceResponse, rmcp::ErrorData> {
        self.read_selected_resource(request, context)
            .await
            .map(rmcp::model::ReadResourceResponse::Complete)
    }
    async fn list_prompts(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<rmcp::model::ListPromptsResult, rmcp::ErrorData> {
        Ok(rmcp::model::ListPromptsResult::with_all_items(
            extensions::catalog_projection::selected_prompts(
                self.conversation_extensions.as_deref(),
            )
            .into_iter()
            .map(|binding| binding.declaration)
            .collect(),
        ))
    }
    async fn get_prompt(
        &self,
        request: rmcp::model::GetPromptRequestParams,
        context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<rmcp::model::GetPromptResponse, rmcp::ErrorData> {
        self.get_selected_prompt(request, context)
            .await
            .map(rmcp::model::GetPromptResponse::Complete)
    }

    fn get_info(&self) -> rmcp::model::ServerInfo {
        let scope = self
            .space_scope
            .as_deref()
            .unwrap_or("Use list_spaces to select an authorized Space");
        rmcp::model::ServerInfo::new(
            rmcp::model::ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .enable_prompts()
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

pub(crate) fn local_tool_names() -> Vec<String> {
    SpaceMcpServer::tool_router()
        .list_all()
        .into_iter()
        .map(|tool| tool.name.into_owned())
        .collect()
}
