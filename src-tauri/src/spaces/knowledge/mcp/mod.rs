use rmcp::{handler::server::wrapper::Parameters, tool, tool_router, ErrorData as McpError};
use serde::Serialize;
use serde_json::json;
use tauri::AppHandle;

pub(crate) mod agent_server;
mod arguments;
mod query;
mod relation_path;
mod transport;

use arguments::{CommitKnowledgeArgs, HistoryArgs, NodeArgs, PathArgs, ProvenanceArgs, SearchArgs};

use super::{change_commit::KnowledgeChange, changes};

#[derive(Clone)]
struct SpaceMcpServer {
    app: AppHandle,
    space_scope: Option<String>,
    actor_id: String,
}

#[tool_router(server_handler)]
impl SpaceMcpServer {
    #[tool(description = "List Knovum Spaces available to the local knowledge server")]
    async fn list_spaces(&self) -> Result<String, McpError> {
        query::list_spaces(&self.app, self.space_scope.as_deref()).await
    }

    #[tool(
        description = "Search active Knovum knowledge nodes in one Space. Returns source-grounded objects and resources, not chat history."
    )]
    async fn search_knowledge(
        &self,
        Parameters(args): Parameters<SearchArgs>,
    ) -> Result<String, McpError> {
        self.require_space(&args.space_id)?;
        query::search_knowledge(&self.app, args).await
    }

    #[tool(description = "Get one active Knovum knowledge node by id inside a Space")]
    async fn get_node(&self, Parameters(args): Parameters<NodeArgs>) -> Result<String, McpError> {
        self.require_space(&args.space_id)?;
        query::get_node(&self.app, args).await
    }

    #[tool(
        description = "Get provenance for a Knovum node or relation, including source locator, source revision, quote, status, and actor"
    )]
    async fn get_provenance(
        &self,
        Parameters(args): Parameters<ProvenanceArgs>,
    ) -> Result<String, McpError> {
        self.require_space(&args.space_id)?;
        query::get_provenance(&self.app, args).await
    }

    #[tool(description = "Read temporal knowledge history for a Space or one knowledge entity")]
    async fn get_history(
        &self,
        Parameters(args): Parameters<HistoryArgs>,
    ) -> Result<String, McpError> {
        self.require_space(&args.space_id)?;
        query::get_history(&self.app, args).await
    }

    #[tool(
        description = "Find the shortest active/contested semantic relation path between two nodes in one Space"
    )]
    async fn find_relation_path(
        &self,
        Parameters(args): Parameters<PathArgs>,
    ) -> Result<String, McpError> {
        self.require_space(&args.space_id)?;
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

    #[tool(
        description = "Validate and atomically commit a Knovum Knowledge Change Set. All operations succeed together or the entire change is rolled back."
    )]
    async fn commit_knowledge_changes(
        &self,
        Parameters(args): Parameters<CommitKnowledgeArgs>,
    ) -> Result<String, McpError> {
        self.require_space(&args.space_id)?;
        let operations: Vec<KnowledgeChange> =
            serde_json::from_value(args.operations).map_err(mcp_error)?;
        let result = changes::commit_knowledge_changes(
            self.app.clone(),
            changes::KnowledgeChangeSet::new(args.space_id, self.actor_id.clone(), operations),
        )
        .await
        .map_err(mcp_error)?;
        json_text(json!({"status": "committed", "result": result}))
    }
}

impl SpaceMcpServer {
    fn require_space(&self, requested: &str) -> Result<(), McpError> {
        if self
            .space_scope
            .as_deref()
            .is_none_or(|scope| scope == requested)
        {
            Ok(())
        } else {
            Err(McpError::invalid_params(
                "This Agent session is scoped to a different Knovum Space",
                None,
            ))
        }
    }
}

fn json_text(value: impl Serialize) -> Result<String, McpError> {
    serde_json::to_string_pretty(&value).map_err(|e| mcp_error(e.to_string()))
}

fn mcp_sql_error(error: sqlx::Error) -> McpError {
    mcp_error(error.to_string())
}
fn mcp_error(error: impl ToString) -> McpError {
    McpError::internal_error(error.to_string(), None)
}
