mod search;

use search::append_json_in_filter;

use serde::Deserialize;
use serde_json::Value;
use sqlx::Row;
use tauri::{AppHandle, Runtime};

use crate::spaces::{db, knowledge::search_plan};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeQueryRequest {
    space_id: String,
    node_ids: Option<Vec<String>>,
    current_only: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalKnowledgeSearchRequest {
    query: String,
}

#[tauri::command]
pub async fn list_knowledge_chunks<R: Runtime>(
    app: AppHandle<R>,
    request: KnowledgeQueryRequest,
) -> Result<Vec<Value>, String> {
    let pool = db::pool(&app).await?;
    let source = if request.current_only {
        "spaces_current_knowledge_chunks"
    } else {
        "spaces_knowledge_chunks"
    };
    let mut sql = format!(
        "SELECT c.id, c.space_id, c.node_id, c.origin, c.kind, c.source_revision, c.text, c.locator, c.metadata, c.created_at, c.updated_at \
         FROM {source} c JOIN spaces_nodes n ON n.id = c.node_id AND n.space_id = c.space_id \
         WHERE c.space_id = ? AND n.deleted_at IS NULL",
    );
    append_json_in_filter(&mut sql, "c.node_id", request.node_ids.is_some());
    sql.push_str(" ORDER BY c.created_at");
    let mut query = sqlx::query(&sql).bind(request.space_id);
    if let Some(node_ids) = request.node_ids {
        query = query.bind(serde_json::to_string(&node_ids).map_err(db::json_error)?);
    }
    let rows = query.fetch_all(pool).await.map_err(db::sql_error)?;
    rows.into_iter().map(chunk_row).collect()
}

pub(crate) async fn search_space<R: Runtime>(
    app: &AppHandle<R>,
    space_id: &str,
    query: &str,
    node_ids: Option<Vec<String>>,
) -> Result<Vec<Value>, String> {
    let plan = search_plan::build(query)?;
    if plan.terms.is_empty() {
        return Ok(Vec::new());
    }
    let pool = db::pool(app).await?;
    search::space(
        pool,
        space_id.to_owned(),
        plan.fts_query,
        plan.terms,
        node_ids,
    )
    .await
}

#[tauri::command]
pub async fn search_knowledge_global<R: Runtime>(
    app: AppHandle<R>,
    request: GlobalKnowledgeSearchRequest,
) -> Result<Vec<Value>, String> {
    search_global(&app, &request.query).await
}

pub(crate) async fn search_global<R: Runtime>(
    app: &AppHandle<R>,
    query: &str,
) -> Result<Vec<Value>, String> {
    let plan = search_plan::build(query)?;
    if plan.terms.is_empty() {
        return Ok(Vec::new());
    }
    let pool = db::pool(app).await?;
    search::global(pool, plan.fts_query, plan.terms).await
}

pub(crate) fn chunk_row(row: sqlx::sqlite::SqliteRow) -> Result<Value, String> {
    let locator: String = row.try_get("locator").map_err(db::sql_error)?;
    let metadata: String = row.try_get("metadata").map_err(db::sql_error)?;
    Ok(serde_json::json!({
        "id": row.try_get::<String, _>("id").map_err(db::sql_error)?,
        "spaceId": row.try_get::<String, _>("space_id").map_err(db::sql_error)?,
        "nodeId": row.try_get::<String, _>("node_id").map_err(db::sql_error)?,
        "origin": row.try_get::<String, _>("origin").map_err(db::sql_error)?,
        "kind": row.try_get::<String, _>("kind").map_err(db::sql_error)?,
        "sourceRevision": row.try_get::<String, _>("source_revision").map_err(db::sql_error)?,
        "text": row.try_get::<String, _>("text").map_err(db::sql_error)?,
        "locator": serde_json::from_str::<Value>(&locator).map_err(db::json_error)?,
        "metadata": serde_json::from_str::<Value>(&metadata).map_err(db::json_error)?,
        "createdAt": row.try_get::<i64, _>("created_at").map_err(db::sql_error)?,
        "updatedAt": row.try_get::<i64, _>("updated_at").map_err(db::sql_error)?,
    }))
}
