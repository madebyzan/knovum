use super::super::{support::mcp_error, SpaceMcpServer};
use rmcp::{
    model::{
        GetPromptRequestParams, GetPromptResult, ReadResourceRequestParams, ReadResourceResult,
    },
    service::RequestContext,
    ErrorData, RoleServer,
};
use serde_json::json;

impl SpaceMcpServer {
    pub(in crate::spaces::mcp) async fn read_selected_resource(
        &self,
        request: ReadResourceRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, ErrorData> {
        let turn = self.turns.current()?;
        let bindings = super::catalog_projection::selected_resources(&turn.extensions);
        let binding = bindings
            .iter()
            .find(|binding| binding.declaration.uri == request.uri)
            .ok_or_else(|| mcp_error("resource not selected for this task"))?;
        let value = super::proxy::invoke_value(
            self,
            &binding.tool.name,
            json!({"uri":binding.uri}),
            context,
        )
        .await?;
        if value["isError"] == true {
            return Err(mcp_error(
                "selected resource read failed; inspect the conversation receipt",
            ));
        }
        serde_json::from_value(value).map_err(|error| mcp_error(error.to_string()))
    }
    pub(in crate::spaces::mcp) async fn get_selected_prompt(
        &self,
        request: GetPromptRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, ErrorData> {
        let turn = self.turns.current()?;
        let bindings = super::catalog_projection::selected_prompts(&turn.extensions);
        let binding = bindings
            .iter()
            .find(|binding| binding.declaration.name == request.name)
            .ok_or_else(|| mcp_error("prompt not selected for this task"))?;
        let value = super::proxy::invoke_value(
            self,
            &binding.tool.name,
            json!({"name":binding.name,"arguments":request.arguments.unwrap_or_default()}),
            context,
        )
        .await?;
        if value["isError"] == true {
            return Err(mcp_error(
                "selected prompt retrieval failed; inspect the conversation receipt",
            ));
        }
        serde_json::from_value(value).map_err(|error| mcp_error(error.to_string()))
    }
}
