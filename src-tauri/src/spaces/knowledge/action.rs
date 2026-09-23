use serde_json::Value;
use sqlx::{Sqlite, Transaction};

use crate::spaces::{authorization::Principal, db};

use super::change_commit::{self, KnowledgeChange};
use super::search_plan;

pub(crate) async fn commit_changes(
    tx: &mut Transaction<'_, Sqlite>,
    principal: &Principal,
    space_id: &str,
    parameters: &Value,
    now: i64,
) -> Result<Value, String> {
    let operations = parameters
        .get("operations")
        .cloned()
        .ok_or("commit-knowledge-changes requires operations")?;
    let operations: Vec<KnowledgeChange> =
        serde_json::from_value(operations).map_err(db::json_error)?;
    change_commit::validate_structure(&operations)?;
    let exists = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM spaces WHERE id=$1")
        .bind(space_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    if exists != 1 {
        return Err("Knowledge Change Set target Space does not exist".into());
    }
    change_commit::validate_references(tx, space_id, &operations).await?;
    let actor = format!("{}:{}", principal.kind.as_str(), principal.id);
    change_commit::apply(tx, space_id, &actor, operations, now).await
}

pub(crate) async fn search(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    parameters: &Value,
) -> Result<Value, String> {
    use sqlx::Row;
    let query = parameters
        .get("query")
        .and_then(Value::as_str)
        .ok_or("search-knowledge requires query")?;
    let limit = parameters
        .get("limit")
        .and_then(Value::as_u64)
        .and_then(|value| i64::try_from(value).ok())
        .unwrap_or(20)
        .clamp(1, 100);
    let plan = search_plan::build(query)?;
    let Some(fts_query) = plan.fts_query else {
        return Err("search-query-invalid: query has no searchable terms".into());
    };
    let rows = sqlx::query(
        "SELECT n.id,n.kind,json_extract(n.data,'$.title') title,\
         snippet(spaces_knowledge_fts,3,'‹','›',' … ',28) preview,c.source_revision,c.locator \
         FROM spaces_knowledge_fts JOIN spaces_current_knowledge_chunks c \
         ON c.id=spaces_knowledge_fts.chunk_id JOIN spaces_semantic_nodes n \
         ON n.id=c.node_id AND n.space_id=c.space_id WHERE spaces_knowledge_fts.space_id=$1 \
         AND spaces_knowledge_fts MATCH $2 ORDER BY bm25(spaces_knowledge_fts) LIMIT $3",
    )
    .bind(space_id)
    .bind(fts_query)
    .bind(limit)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    Ok(Value::Array(rows.into_iter().map(|row| serde_json::json!({
        "nodeId": row.get::<String,_>("id"), "kind": row.get::<String,_>("kind"),
        "title": row.get::<Option<String>,_>("title"), "preview": row.get::<String,_>("preview"),
        "sourceRevision": row.get::<String,_>("source_revision"),
        "locator": serde_json::from_str::<Value>(&row.get::<String,_>("locator")).unwrap_or(Value::Null),
    })).collect()))
}

pub(crate) async fn read_node(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    parameters: &Value,
) -> Result<Value, String> {
    use sqlx::Row;
    let node_id = parameters
        .get("nodeId")
        .and_then(Value::as_str)
        .ok_or("read-node requires nodeId")?;
    let row = sqlx::query(
        "SELECT id,kind,data,updated_at FROM spaces_semantic_nodes WHERE space_id=$1 AND id=$2",
    )
    .bind(space_id)
    .bind(node_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)?
    .ok_or("node-unavailable: node does not exist")?;
    let data: String = row.get("data");
    Ok(serde_json::json!({
        "id": row.get::<String,_>("id"), "kind": row.get::<String,_>("kind"),
        "data": serde_json::from_str::<Value>(&data).map_err(db::json_error)?,
        "updatedAt": row.get::<i64,_>("updated_at"),
    }))
}
