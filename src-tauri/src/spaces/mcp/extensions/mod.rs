pub(in crate::spaces::mcp) mod catalog_projection;
mod proxy;
mod resource_requests;
mod skills;
use super::SpaceMcpServer;
use rmcp::{
    model::{CallToolRequestParams, CallToolResponse, ListToolsResult, Tool},
    service::RequestContext,
    ErrorData, RoleServer,
};

impl SpaceMcpServer {
    pub(super) fn extension_tools(&self) -> Vec<Tool> {
        self.conversation_extensions
            .as_ref()
            .map(|snapshot| {
                snapshot
                    .tools
                    .iter()
                    .map(|tool| {
                        Tool::new(
                            tool.name.clone(),
                            tool.description.clone(),
                            std::sync::Arc::new(
                                tool.input_schema
                                    .as_object()
                                    .expect("validated tool input schema")
                                    .clone(),
                            ),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
    pub(super) async fn dispatch_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        if self
            .conversation_extensions
            .as_ref()
            .is_some_and(|snapshot| snapshot.tools.iter().any(|tool| tool.name == request.name))
        {
            return proxy::invoke(self, request, context).await;
        }
        let call = rmcp::handler::server::tool::ToolCallContext::new(self, request, context);
        Self::tool_router().call(call).await
    }
    pub(super) fn tool_listing(&self) -> ListToolsResult {
        let mut tools = Self::tool_router().list_all();
        tools.extend(self.extension_tools());
        ListToolsResult::with_all_items(tools)
    }
}

#[cfg(test)]
#[path = "tests/catalog_projection.rs"]
mod catalog_projection_tests;
