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
    let actor = super::events::EventActor::new(
        principal.kind.event_actor_kind(),
        Some(principal.id.as_str()),
    );
    change_commit::apply(tx, space_id, actor, operations, now).await
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
    Ok(Value::Array(rows.into_iter().map(|row| Ok(serde_json::json!({
        "nodeId": row.try_get::<String,_>("id").map_err(db::sql_error)?, "kind": row.try_get::<String,_>("kind").map_err(db::sql_error)?,
        "title": row.try_get::<Option<String>,_>("title").map_err(db::sql_error)?, "preview": row.try_get::<String,_>("preview").map_err(db::sql_error)?,
        "sourceRevision": row.try_get::<String,_>("source_revision").map_err(db::sql_error)?,
        "locator": serde_json::from_str::<Value>(&row.try_get::<String,_>("locator").map_err(db::sql_error)?).map_err(db::json_error)?,
    }))).collect::<Result<Vec<_>, String>>()?))
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
    let data: String = row.try_get("data").map_err(db::sql_error)?;
    Ok(serde_json::json!({
        "id": row.try_get::<String,_>("id").map_err(db::sql_error)?, "kind": row.try_get::<String,_>("kind").map_err(db::sql_error)?,
        "data": serde_json::from_str::<Value>(&data).map_err(db::json_error)?,
        "updatedAt": row.try_get::<i64,_>("updated_at").map_err(db::sql_error)?,
    }))
}
