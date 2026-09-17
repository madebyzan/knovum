use rmcp::ErrorData as McpError;
use serde_json::json;
use sqlx::Row;
use tauri::AppHandle;

use super::super::{json_text, mcp_error, mcp_sql_error};
use crate::spaces::db;
use crate::spaces::knowledge::mcp::arguments::SearchArgs;
use crate::spaces::knowledge::search_plan;

pub(super) async fn run(app: &AppHandle, args: SearchArgs) -> Result<String, McpError> {
    if args.query.trim().is_empty() {
        return Err(McpError::invalid_params("query must be non-empty", None));
    }
    let pool = db::pool(app).await.map_err(mcp_error)?;
    let limit = args.limit.clamp(1, 100);
    let plan = search_plan::build(args.query.trim())
        .map_err(|error| McpError::invalid_params(error, None))?;
    if plan.terms.is_empty() {
        return Err(McpError::invalid_params(
            "query must contain searchable text",
            None,
        ));
    }
    let rows = if let Some(fts_query) = plan.fts_query {
        sqlx::query(
            "SELECT spaces_knowledge_fts.node_id AS id,n.kind,json_extract(n.data,'$.title') AS title, \
                 snippet(spaces_knowledge_fts,3,'‹','›',' … ',28) AS preview,bm25(spaces_knowledge_fts) AS rank,n.updated_at \
                 FROM spaces_knowledge_fts \
                 JOIN spaces_current_knowledge_chunks c ON c.id=spaces_knowledge_fts.chunk_id \
                 JOIN spaces_semantic_nodes n ON n.id=c.node_id AND n.space_id=c.space_id \
                 WHERE spaces_knowledge_fts.space_id=$1 AND spaces_knowledge_fts MATCH $2 \
                 ORDER BY rank LIMIT $3",
        )
        .bind(&args.space_id)
        .bind(fts_query)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(mcp_sql_error)?
    } else {
        let mut sql = String::from(
            "SELECT c.node_id AS id,n.kind,json_extract(n.data,'$.title') AS title,c.text AS preview,-1.0 AS rank,n.updated_at \
                 FROM spaces_current_knowledge_chunks c JOIN spaces_semantic_nodes n ON n.id=c.node_id AND n.space_id=c.space_id \
                 WHERE c.space_id=?",
        );
        for _ in &plan.terms {
            sql.push_str(" AND instr(lower(c.text), lower(?)) > 0");
        }
        sql.push_str(" ORDER BY c.updated_at DESC LIMIT ?");
        let mut query = sqlx::query(&sql).bind(&args.space_id);
        for term in plan.terms {
            query = query.bind(term);
        }
        query
            .bind(limit)
            .fetch_all(pool)
            .await
            .map_err(mcp_sql_error)?
    };
    let values = rows
        .into_iter()
        .map(|row| {
            json!({
                "id": row.get::<String,_>("id"),
                "kind": row.get::<String,_>("kind"),
                "title": row.get::<Option<String>,_>("title"),
                "preview": row.get::<String,_>("preview"),
                "rank": row.get::<f64,_>("rank"),
                "updatedAt": row.get::<i64,_>("updated_at"),
            })
        })
        .collect::<Vec<_>>();
    json_text(values)
}
