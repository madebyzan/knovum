use rmcp::{handler::server::wrapper::Parameters, tool, tool_router, ErrorData as McpError};
use serde_json::json;

use super::{
    arguments::{
        AssetArgs, ContentsArgs, HistoryArgs, NodeArgs, NodeChunksArgs, PathArgs, ProvenanceArgs,
        RelationsArgs, SearchArgs, SpaceArgs,
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
        description = "List current canvas resources with summaries, tags, object types, and chunk counts (including unindexed images, videos, files and notes). Start with offset=0 and limit=40; use nextOffset for further pages.",
        annotations(read_only_hint = true)
    )]
    async fn list_space_contents(
        &self,
        Parameters(args): Parameters<ContentsArgs>,
    ) -> Result<String, McpError> {
        self.require_read(&args.space_id).await?;
        query::list_space_contents(&self.app, args).await
    }

    #[tool(
        description = "Hybrid semantic and keyword search over indexed Space content. Supports '|' OR branches (e.g. 'pricing | revenue'), optional node_ids/roles filters, and mode='nodes' for a compact per-node hit survey before drilling into passages (default mode='passages' returns compact excerpts, locators, and cite ids).",
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
        description = "Read one current canvas node by ID with compact semantic data, summary/chapters, totalChunks, chunkRoles, and the first 20 passages in natural reading order (each with its cite id). Use read_node_chunks to paginate remaining passages instead of repeated searches.",
        annotations(read_only_hint = true)
    )]
    async fn get_node(&self, Parameters(args): Parameters<NodeArgs>) -> Result<String, McpError> {
        self.require_read(&args.space_id).await?;
        query::get_node(&self.app, &self.turns.current()?.run_id, args).await
    }

    #[tool(
        description = "Read a node's indexed passages in natural reading order (startMs, page, slide, line, or document order), optionally filtered by role. Paginate via offset/limit, or pass around_chunk_id (with context_chunks, default 3) to jump directly to the surrounding context of a search hit. Every passage returns its cite id.",
        annotations(read_only_hint = true)
    )]
    async fn read_node_chunks(
        &self,
        Parameters(args): Parameters<NodeChunksArgs>,
    ) -> Result<String, McpError> {
        self.require_read(&args.space_id).await?;
        query::read_node_chunks(&self.app, &self.turns.current()?.run_id, args).await
    }

    #[tool(
        description = "List system and custom structured knowledge object types (Claim, Decision, Question, Evidence, Concept, Experiment), their required fields, valid statuses, and relation kinds for commit_knowledge_changes.",
        annotations(read_only_hint = true)
    )]
    async fn list_object_types(
        &self,
        Parameters(args): Parameters<SpaceArgs>,
    ) -> Result<String, McpError> {
        self.require_read(&args.space_id).await?;
        query::list_object_types(&self.app, &args.space_id).await
    }

    #[tool(
        description = "List active and contested knowledge relations in the Space, optionally filtered by node_id or relation kind.",
        annotations(read_only_hint = true)
    )]
    async fn list_relations(
        &self,
        Parameters(args): Parameters<RelationsArgs>,
    ) -> Result<String, McpError> {
        self.require_read(&args.space_id).await?;
        query::list_relations(&self.app, args).await
    }

    #[tool(
        description = "Resolve a canvas node at its exact asset_revision (get_node.assetRevision) to a readable local source file. For images this renders current crop/rotation/adjustments. Use the returned path with available image or file tools, then save a new result in the output workspace.",
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
        description = "Find the shortest active or contested relation path between two nodes",
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
