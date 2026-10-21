use serde_json::{json, Value};
use sqlx::{sqlite::SqliteRow, Row, SqliteConnection};

use crate::spaces::{db, knowledge::search_plan};

pub(crate) async fn search_knowledge(
    connection: &mut SqliteConnection,
    space_id: &str,
    query: &str,
    limit: i64,
) -> Result<Value, String> {
    if query.trim().is_empty() {
        return Err("invalid-query: query must be non-empty".into());
    }
    let plan = search_plan::build(query.trim())?;
    if plan.terms.is_empty() {
        return Err("invalid-query: query must contain searchable text".into());
    }
    let rows = if let Some(fts_query) = plan.fts_query {
        sqlx::query(FTS_QUERY)
            .bind(space_id)
            .bind(fts_query)
            .bind(limit.clamp(1, 100))
            .fetch_all(&mut *connection)
            .await
            .map_err(db::sql_error)?
    } else {
        search_substring(connection, space_id, plan.terms, limit.clamp(1, 100)).await?
    };
    Ok(Value::Array(
        rows.into_iter()
            .map(decode_search_projection)
            .collect::<Result<Vec<_>, _>>()?,
    ))
}

const FTS_QUERY: &str = "SELECT spaces_knowledge_fts.node_id AS id,n.kind,\
 json_extract(n.data,'$.title') AS title,snippet(spaces_knowledge_fts,3,'‹','›',' … ',28) AS preview,\
 bm25(spaces_knowledge_fts) AS rank,c.id AS chunk_id,c.source_revision,c.locator,n.updated_at FROM spaces_knowledge_fts \
 JOIN spaces_current_knowledge_chunks c ON c.id=spaces_knowledge_fts.chunk_id \
 JOIN spaces_semantic_nodes n ON n.id=c.node_id AND n.space_id=c.space_id \
 WHERE spaces_knowledge_fts.space_id=$1 AND spaces_knowledge_fts MATCH $2 ORDER BY rank LIMIT $3";

async fn search_substring(
    connection: &mut SqliteConnection,
    space_id: &str,
    terms: Vec<String>,
    limit: i64,
) -> Result<Vec<SqliteRow>, String> {
    let mut sql = sqlx::QueryBuilder::<sqlx::Sqlite>::new(
        "SELECT c.node_id AS id,n.kind,json_extract(n.data,'$.title') AS title,\
         c.text AS preview,-1.0 AS rank,c.id AS chunk_id,c.source_revision,c.locator,n.updated_at FROM spaces_current_knowledge_chunks c \
         JOIN spaces_semantic_nodes n ON n.id=c.node_id AND n.space_id=c.space_id WHERE c.space_id=",
    );
    sql.push_bind(space_id);
    for term in terms {
        sql.push(" AND instr(lower(c.text), lower(")
            .push_bind(term)
            .push(")) > 0");
    }
    sql.push(" ORDER BY c.updated_at DESC LIMIT ")
        .push_bind(limit);
    sql.build()
        .fetch_all(connection)
        .await
        .map_err(db::sql_error)
}

fn decode_search_projection(row: SqliteRow) -> Result<Value, String> {
    Ok(json!({
        "nodeId": row.try_get::<String,_>("id").map_err(db::sql_error)?, "kind": row.try_get::<String,_>("kind").map_err(db::sql_error)?,
        "title": row.try_get::<Option<String>,_>("title").map_err(db::sql_error)?, "preview": row.try_get::<String,_>("preview").map_err(db::sql_error)?,
        "rank": row.try_get::<f64,_>("rank").map_err(db::sql_error)?, "updatedAt": row.try_get::<i64,_>("updated_at").map_err(db::sql_error)?,
        "chunkId": row.try_get::<String,_>("chunk_id").map_err(db::sql_error)?,
        "sourceRevision": row.try_get::<String,_>("source_revision").map_err(db::sql_error)?,
        "locator": serde_json::from_str::<Value>(&row.try_get::<String,_>("locator").map_err(db::sql_error)?).map_err(db::json_error)?,
    }))
}
