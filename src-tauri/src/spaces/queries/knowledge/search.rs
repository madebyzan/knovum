use serde_json::Value;
use sqlx::Row;

use crate::spaces::db;

pub(super) async fn search_space(
    pool: &sqlx::SqlitePool,
    space_id: String,
    fts_query: Option<String>,
    terms: Vec<String>,
    node_ids: Option<Vec<String>>,
) -> Result<Vec<Value>, String> {
    if let Some(fts_query) = fts_query {
        let fts_rows = search_fts(pool, space_id.clone(), fts_query, node_ids.clone()).await?;
        if !fts_rows.is_empty() || terms.is_empty() {
            return Ok(fts_rows);
        }
    }
    search_substring(pool, space_id, terms, node_ids).await
}

pub(super) async fn search_global(
    pool: &sqlx::SqlitePool,
    fts_query: Option<String>,
    terms: Vec<String>,
) -> Result<Vec<Value>, String> {
    if let Some(fts_query) = fts_query {
        let rows = sqlx::query(
            "SELECT c.id AS chunk_id, c.node_id, n.kind AS node_kind, \
             json_extract(n.data, '$.title') AS node_title, c.kind AS chunk_kind, c.source_revision, c.locator, c.text, c.metadata, \
             snippet(spaces_knowledge_fts, 3, '‹', '›', ' … ', 28) AS snippet, \
             bm25(spaces_knowledge_fts) AS rank, s.id AS space_id, s.title AS space_title \
             FROM spaces_knowledge_fts \
             JOIN spaces_current_knowledge_chunks c ON c.id = spaces_knowledge_fts.chunk_id \
             JOIN spaces_nodes n ON n.id = c.node_id AND n.space_id = c.space_id \
             JOIN spaces s ON s.id = c.space_id \
             WHERE s.archived_at IS NULL AND n.deleted_at IS NULL AND spaces_knowledge_fts MATCH ? \
             ORDER BY rank LIMIT 80",
        )
        .bind(fts_query)
        .fetch_all(pool)
        .await
        .map_err(db::sql_error)?;
        if !rows.is_empty() || terms.is_empty() {
            return rows
                .into_iter()
                .map(decode_global_search_projection)
                .collect();
        }
    }
    let mut sql = sqlx::QueryBuilder::<sqlx::Sqlite>::new(
        "SELECT c.id AS chunk_id, c.node_id, n.kind AS node_kind, json_extract(n.data, '$.title') AS node_title, \
         c.kind AS chunk_kind, c.source_revision, c.locator, c.text, c.metadata, c.text AS snippet, -1.0 AS rank, \
         s.id AS space_id, s.title AS space_title \
         FROM spaces_current_knowledge_chunks c \
         JOIN spaces_nodes n ON n.id = c.node_id AND n.space_id = c.space_id \
         JOIN spaces s ON s.id = c.space_id \
         WHERE s.archived_at IS NULL AND n.deleted_at IS NULL",
    );
    append_terms(&mut sql, terms);
    sql.push(" ORDER BY c.updated_at DESC LIMIT 80");
    let rows = sql.build().fetch_all(pool).await.map_err(db::sql_error)?;
    rows.into_iter()
        .map(decode_global_search_projection)
        .collect()
}

async fn search_fts(
    pool: &sqlx::SqlitePool,
    space_id: String,
    fts_query: String,
    node_ids: Option<Vec<String>>,
) -> Result<Vec<Value>, String> {
    let mut sql = sqlx::QueryBuilder::<sqlx::Sqlite>::new(
        "SELECT c.id AS chunk_id, c.node_id, n.kind AS node_kind, json_extract(n.data, '$.title') AS node_title, \
         c.kind AS chunk_kind, c.source_revision, c.locator, c.text, c.metadata, \
         snippet(spaces_knowledge_fts, 3, '‹', '›', ' … ', 28) AS snippet, bm25(spaces_knowledge_fts) AS rank \
         FROM spaces_knowledge_fts \
         JOIN spaces_current_knowledge_chunks c ON c.id = spaces_knowledge_fts.chunk_id \
         JOIN spaces_nodes n ON n.id = c.node_id AND n.space_id = c.space_id \
         WHERE spaces_knowledge_fts.space_id = ",
    );
    sql.push_bind(space_id)
        .push(" AND n.deleted_at IS NULL AND spaces_knowledge_fts MATCH ")
        .push_bind(fts_query);
    append_json_in_filter(&mut sql, node_ids)?;
    sql.push(" ORDER BY rank LIMIT 80");
    let rows = sql.build().fetch_all(pool).await.map_err(db::sql_error)?;
    rows.into_iter()
        .map(decode_space_search_projection)
        .collect()
}

async fn search_substring(
    pool: &sqlx::SqlitePool,
    space_id: String,
    terms: Vec<String>,
    node_ids: Option<Vec<String>>,
) -> Result<Vec<Value>, String> {
    let mut sql = sqlx::QueryBuilder::<sqlx::Sqlite>::new(
        "SELECT c.id AS chunk_id, c.node_id, n.kind AS node_kind, json_extract(n.data, '$.title') AS node_title, \
         c.kind AS chunk_kind, c.source_revision, c.locator, c.text, c.metadata, c.text AS snippet, -1.0 AS rank \
         FROM spaces_current_knowledge_chunks c JOIN spaces_nodes n ON n.id = c.node_id AND n.space_id = c.space_id \
         WHERE c.space_id = ",
    );
    sql.push_bind(space_id).push(" AND n.deleted_at IS NULL");
    append_terms(&mut sql, terms);
    append_json_in_filter(&mut sql, node_ids)?;
    sql.push(" ORDER BY c.updated_at DESC LIMIT 80");
    let rows = sql.build().fetch_all(pool).await.map_err(db::sql_error)?;
    rows.into_iter()
        .map(decode_space_search_projection)
        .collect()
}

fn decode_global_search_projection(row: sqlx::sqlite::SqliteRow) -> Result<Value, String> {
    decode_search_projection(row, true)
}

fn decode_space_search_projection(row: sqlx::sqlite::SqliteRow) -> Result<Value, String> {
    decode_search_projection(row, false)
}

fn decode_search_projection(
    row: sqlx::sqlite::SqliteRow,
    include_space_metadata: bool,
) -> Result<Value, String> {
    let locator: String = row.try_get("locator").map_err(db::sql_error)?;
    let metadata: String = row.try_get("metadata").map_err(db::sql_error)?;
    let rank: f64 = row.try_get("rank").map_err(db::sql_error)?;
    let node_title = row
        .try_get::<Option<String>, _>("node_title")
        .map_err(db::sql_error)?
        .ok_or_else(|| "Knowledge node is missing its required title".to_owned())?;
    let mut value = serde_json::json!({
        "chunkId": row.try_get::<String, _>("chunk_id").map_err(db::sql_error)?,
        "nodeId": row.try_get::<String, _>("node_id").map_err(db::sql_error)?,
        "nodeKind": row.try_get::<String, _>("node_kind").map_err(db::sql_error)?,
        "nodeTitle": node_title,
        "chunkKind": row.try_get::<String, _>("chunk_kind").map_err(db::sql_error)?,
        "sourceRevision": row.try_get::<String, _>("source_revision").map_err(db::sql_error)?,
        "locator": serde_json::from_str::<Value>(&locator).map_err(db::json_error)?,
        "metadata": serde_json::from_str::<Value>(&metadata).map_err(db::json_error)?,
        "snippet": row.try_get::<String, _>("snippet").map_err(db::sql_error)?,
        "text": row.try_get::<String, _>("text").map_err(db::sql_error)?,
        "score": -rank,
    });
    if include_space_metadata {
        let object = value
            .as_object_mut()
            .expect("search result must be an object");
        object.insert(
            "spaceId".into(),
            Value::from(
                row.try_get::<String, _>("space_id")
                    .map_err(db::sql_error)?,
            ),
        );
        object.insert(
            "spaceTitle".into(),
            Value::from(
                row.try_get::<String, _>("space_title")
                    .map_err(db::sql_error)?,
            ),
        );
    }
    Ok(value)
}

fn append_terms(sql: &mut sqlx::QueryBuilder<sqlx::Sqlite>, terms: Vec<String>) {
    for term in terms {
        sql.push(" AND instr(lower(c.text), lower(")
            .push_bind(term)
            .push(")) > 0");
    }
}

pub(super) fn append_json_in_filter(
    sql: &mut sqlx::QueryBuilder<sqlx::Sqlite>,
    node_ids: Option<Vec<String>>,
) -> Result<(), String> {
    if let Some(node_ids) = node_ids {
        sql.push(" AND c.node_id IN (SELECT value FROM json_each(")
            .push_bind(sqlx::types::Json(node_ids))
            .push("))");
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests/search.rs"]
mod tests;
