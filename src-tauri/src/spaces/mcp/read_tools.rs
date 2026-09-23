use rmcp::{handler::server::wrapper::Parameters, tool, tool_router, ErrorData as McpError};
use serde_json::json;

use super::{
    arguments::{HistoryArgs, NodeArgs, PathArgs, ProvenanceArgs, SearchArgs},
    query, relation_path,
    support::json_text,
    SpaceMcpServer,
};

#[tool_router(router = read_router, vis = "pub(super)")]
impl SpaceMcpServer {
    #[tool(description = "List Knovum Spaces available to the local knowledge server")]
    async fn list_spaces(&self) -> Result<String, McpError> {
        if let Some(space_id) = self.space_scope.as_deref() {
            self.authorize_read(space_id).await?;
        }
        query::list_spaces(&self.app, self.space_scope.as_deref()).await
    }

    #[tool(description = "Search active knowledge nodes in one authorized Space")]
    async fn search_knowledge(
        &self,
        Parameters(args): Parameters<SearchArgs>,
    ) -> Result<String, McpError> {
        self.require_read(&args.space_id).await?;
        query::search_knowledge(&self.app, args).await
    }

    #[tool(description = "Get one active knowledge node by id inside a Space")]
    async fn get_node(&self, Parameters(args): Parameters<NodeArgs>) -> Result<String, McpError> {
        self.require_read(&args.space_id).await?;
        query::get_node(&self.app, args).await
    }

    #[tool(description = "Get source provenance for one node or relation")]
    async fn get_provenance(
        &self,
        Parameters(args): Parameters<ProvenanceArgs>,
    ) -> Result<String, McpError> {
        self.require_read(&args.space_id).await?;
        query::get_provenance(&self.app, args).await
    }

    #[tool(description = "Read temporal history for a Space or knowledge entity")]
    async fn get_history(
        &self,
        Parameters(args): Parameters<HistoryArgs>,
    ) -> Result<String, McpError> {
        self.require_read(&args.space_id).await?;
        query::get_history(&self.app, args).await
    }

    #[tool(description = "Find the shortest active or contested relation path")]
    async fn find_relation_path(
        &self,
        Parameters(args): Parameters<PathArgs>,
    ) -> Result<String, McpError> {
        self.require_read(&args.space_id).await?;
        let path = relation_path::find(
            &self.app,
            &args.space_id,
            &args.source_id,
            &args.target_id,
            args.max_depth.clamp(1, 12),
        )
        .await?;
        json_text(json!({"path": path}))
    }
}

impl SpaceMcpServer {
    async fn require_read(&self, space_id: &str) -> Result<(), McpError> {
        self.require_space(space_id)?;
        self.authorize_read(space_id).await
    }
}
