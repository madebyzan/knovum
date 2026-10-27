use rmcp::{handler::server::wrapper::Parameters, tool, tool_router, ErrorData as McpError};
use serde_json::json;

use super::{
    arguments::{
        AssetArgs, ContentsArgs, HistoryArgs, NodeArgs, PathArgs, ProvenanceArgs, SearchArgs,
    },
    query, relation_path,
    support::json_text,
    SpaceMcpServer,
};

#[tool_router(router = read_router, vis = "pub(super)")]
impl SpaceMcpServer {
    #[tool(
        description = "List Knovum Spaces available to the local knowledge server",
        annotations(read_only_hint = true)
    )]
    async fn list_spaces(&self) -> Result<String, McpError> {
        if let Some(space_id) = self.space_scope.as_deref() {
            self.authorize_read(space_id).await?;
        }
        query::list_spaces(&self.app, self.space_scope.as_deref()).await
    }

    #[tool(
        description = "List current canvas resources, including unindexed images, videos, files and notes. Start with offset=0 and limit=40; use nextOffset for further pages. Search results are not a canvas inventory.",
        annotations(read_only_hint = true)
    )]
    async fn list_space_contents(
        &self,
        Parameters(args): Parameters<ContentsArgs>,
    ) -> Result<String, McpError> {
        self.require_read(&args.space_id).await?;
        let pool = crate::spaces::db::pool(&self.app)
            .await
            .map_err(super::mcp_error)?;
        let mut tx = pool
            .begin()
            .await
            .map_err(|error| super::mcp_error(error.to_string()))?;
        let page = crate::spaces::queries::knowledge_projection::list_contents(
            &mut tx,
            &args.space_id,
            args.offset,
            args.limit,
        )
        .await
        .map_err(super::mcp_error)?;
        json_text(page)
    }

    #[tool(
        description = "Search indexed Space content by keywords; returns nodeId, sourceRevision, excerpts, locators and the cite id of each passage. Empty results do not mean the canvas is empty; use list_space_contents for inventory.",
        annotations(read_only_hint = true)
    )]
    async fn search_knowledge(
        &self,
        Parameters(args): Parameters<SearchArgs>,
    ) -> Result<String, McpError> {
        self.require_read(&args.space_id).await?;
        query::search_knowledge(&self.app, &self.turns.current()?.run_id, args).await
    }

    #[tool(
        description = "Read one current canvas node by ID with data, indexed chunks (each with its cite id) and managed assetPath. These source assets remain available after output workspace cleanup. Never infer a source is missing from an empty working directory.",
        annotations(read_only_hint = true)
    )]
    async fn get_node(&self, Parameters(args): Parameters<NodeArgs>) -> Result<String, McpError> {
        self.require_read(&args.space_id).await?;
        query::get_node(&self.app, &self.turns.current()?.run_id, args).await
    }

    #[tool(
        description = "Resolve a canvas node at its exact asset_revision (get_node.assetRevision) to a readable local source file. For images this renders current crop/rotation/adjustments. Use the returned path with available image or file tools, then save a new result in the output workspace. No source is copied into the output folder.",
        annotations(read_only_hint = true)
    )]
    async fn get_node_asset(
        &self,
        Parameters(args): Parameters<AssetArgs>,
    ) -> Result<String, McpError> {
        self.node_asset(args).await
    }

    #[tool(
        description = "Get source provenance for one node or relation",
        annotations(read_only_hint = true)
    )]
    async fn list_provenance(
        &self,
        Parameters(args): Parameters<ProvenanceArgs>,
    ) -> Result<String, McpError> {
        self.require_read(&args.space_id).await?;
        query::list_provenance(&self.app, args).await
    }

    #[tool(
        description = "Read temporal history for a Space or knowledge entity",
        annotations(read_only_hint = true)
    )]
    async fn list_knowledge_events(
        &self,
        Parameters(args): Parameters<HistoryArgs>,
    ) -> Result<String, McpError> {
        self.require_read(&args.space_id).await?;
        query::list_knowledge_events(&self.app, args).await
    }

    #[tool(
        description = "Find the shortest active or contested relation path",
        annotations(read_only_hint = true)
    )]
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
    pub(super) async fn require_read(&self, space_id: &str) -> Result<(), McpError> {
        self.require_space(space_id)?;
        self.authorize_read(space_id).await
    }
}
