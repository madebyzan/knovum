use serde_json::{json, Map, Value};
use sqlx::{Row, SqliteConnection};

use crate::spaces::db;

const DEFAULT_CONTEXT_CHUNKS: i64 = 3;
const MAX_CONTEXT_CHUNKS: i64 = 25;

macro_rules! natural_chunk_order {
    () => {
        "CASE json_extract(c.metadata, '$.role') \
           WHEN 'summary' THEN 0 \
           WHEN 'chapter' THEN 1 \
           ELSE 2 \
         END, \
         COALESCE( \
           CAST(json_extract(c.locator, '$.location.startMs') AS INTEGER), \
           CAST(json_extract(c.locator, '$.location.page') AS INTEGER), \
           CAST(json_extract(c.locator, '$.location.slide') AS INTEGER), \
           CAST(json_extract(c.locator, '$.location.lineStart') AS INTEGER), \
           CAST(json_extract(c.metadata, '$.index') AS INTEGER), \
           c.created_at \
         ), \
         COALESCE(CAST(json_extract(c.metadata, '$.part') AS INTEGER), 0), \
         c.id"
    };
}

pub(crate) struct ChunkReadFilter {
    pub(crate) role: Option<String>,
    pub(crate) offset: i64,
    pub(crate) limit: i64,
    pub(crate) around_chunk_id: Option<String>,
    pub(crate) context_chunks: Option<i64>,
}

pub(crate) struct NodeChunksSlice {
    pub(crate) total_chunks: i64,
    pub(crate) next_offset: Option<i64>,
    pub(crate) roles: Map<String, Value>,
    pub(crate) chunks: Vec<Value>,
}

pub(crate) async fn load_node_chunks(
    connection: &mut SqliteConnection,
    space_id: &str,
    node_id: &str,
    filter: ChunkReadFilter,
) -> Result<Value, String> {
    if filter.offset < 0 || !(1..=100).contains(&filter.limit) {
        return Err(
            "node-chunks-invalid: offset must be nonnegative and limit must be 1–100".into(),
        );
    }
    let exists: Option<i64> = sqlx::query_scalar(
        "SELECT 1 FROM spaces_nodes WHERE space_id=$1 AND id=$2 AND deleted_at IS NULL \
         AND kind NOT IN ('credential','portal')",
    )
    .bind(space_id)
    .bind(node_id)
    .fetch_optional(&mut *connection)
    .await
    .map_err(db::sql_error)?;
    if exists.is_none() {
        return Err("node-unavailable: node does not exist in the requested Space".into());
    }

    let normalized_role = filter
        .role
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let anchor = match filter
        .around_chunk_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(chunk_id) => {
            let radius = filter
                .context_chunks
                .unwrap_or(DEFAULT_CONTEXT_CHUNKS)
                .clamp(0, MAX_CONTEXT_CHUNKS);
            let anchor_offset =
                resolve_chunk_offset(connection, space_id, node_id, normalized_role, chunk_id)
                    .await?;
            let window_start = (anchor_offset - radius).max(0);
            let window_end = anchor_offset + radius + 1;
            let window_limit = (window_end - window_start).clamp(1, 100);
            Some((
                chunk_id.to_owned(),
                anchor_offset,
                window_start,
                window_limit,
            ))
        }
        None => None,
    };

    let (effective_offset, effective_limit) = match &anchor {
        Some((_, _, start, limit)) => (*start, *limit),
        None => (filter.offset, filter.limit),
    };
    let effective_filter = ChunkReadFilter {
        role: filter.role,
        offset: effective_offset,
        limit: effective_limit,
        around_chunk_id: None,
        context_chunks: None,
    };
    let slice = query_slice(connection, space_id, node_id, &effective_filter).await?;
    let mut payload = json!({
        "nodeId": node_id,
        "totalChunks": slice.total_chunks,
        "offset": effective_offset,
        "limit": effective_limit,
        "nextOffset": slice.next_offset,
        "chunks": slice.chunks,
    });
    if let Some((anchor_chunk_id, anchor_offset, _, _)) = anchor {
        payload["anchorChunkId"] = Value::String(anchor_chunk_id);
        payload["anchorOffset"] = Value::from(anchor_offset);
    }
    if !slice.roles.is_empty() {
        payload["roles"] = Value::Object(slice.roles);
    }
    Ok(payload)
}

async fn resolve_chunk_offset(
    connection: &mut SqliteConnection,
    space_id: &str,
    node_id: &str,
    role: Option<&str>,
    chunk_id: &str,
) -> Result<i64, String> {
    sqlx::query_scalar(concat!(
        "WITH ordered AS ( \
           SELECT c.id, ROW_NUMBER() OVER (ORDER BY ",
        natural_chunk_order!(),
        ") - 1 AS position \
           FROM spaces_current_knowledge_chunks c \
           JOIN spaces_semantic_nodes n ON n.id=c.node_id AND n.space_id=c.space_id \
           WHERE c.space_id=$1 AND c.node_id=$2 \
             AND ($3 IS NULL OR json_extract(c.metadata,'$.role')=$3) \
         ) \
         SELECT position FROM ordered WHERE id=$4",
    ))
    .bind(space_id)
    .bind(node_id)
    .bind(role)
    .bind(chunk_id)
    .fetch_optional(&mut *connection)
    .await
    .map_err(db::sql_error)?
    .ok_or_else(|| {
        format!(
            "chunk-unavailable: aroundChunkId {chunk_id} does not exist in this node or role filter"
        )
    })
}

pub(super) async fn query_slice(
    connection: &mut SqliteConnection,
    space_id: &str,
    node_id: &str,
    filter: &ChunkReadFilter,
) -> Result<NodeChunksSlice, String> {
    let role = filter
        .role
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let total_chunks: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM spaces_current_knowledge_chunks c \
         JOIN spaces_semantic_nodes n ON n.id=c.node_id AND n.space_id=c.space_id \
         WHERE c.space_id=$1 AND c.node_id=$2 \
           AND ($3 IS NULL OR json_extract(c.metadata,'$.role')=$3)",
    )
    .bind(space_id)
    .bind(node_id)
    .bind(role)
    .fetch_one(&mut *connection)
    .await
    .map_err(db::sql_error)?;

    let roles = load_role_counts(connection, space_id, node_id).await?;
    let rows = sqlx::query(concat!(
        "SELECT c.id, c.kind, c.source_revision, c.text, c.locator, \
           json_extract(c.metadata,'$.role') AS role \
         FROM spaces_current_knowledge_chunks c \
         JOIN spaces_semantic_nodes n ON n.id=c.node_id AND n.space_id=c.space_id \
         WHERE c.space_id=$1 AND c.node_id=$2 \
           AND ($3 IS NULL OR json_extract(c.metadata,'$.role')=$3) \
         ORDER BY ",
        natural_chunk_order!(),
        " LIMIT $4 OFFSET $5",
    ))
    .bind(space_id)
    .bind(node_id)
    .bind(role)
    .bind(filter.limit)
    .bind(filter.offset)
    .fetch_all(&mut *connection)
    .await
    .map_err(db::sql_error)?;

    let chunks = rows
        .into_iter()
        .map(|row| {
            let locator: String = row.try_get("locator").map_err(db::sql_error)?;
            let mut item = json!({
                "id": row.try_get::<String, _>("id").map_err(db::sql_error)?,
                "kind": row.try_get::<String, _>("kind").map_err(db::sql_error)?,
                "sourceRevision": row.try_get::<String, _>("source_revision").map_err(db::sql_error)?,
                "text": row.try_get::<String, _>("text").map_err(db::sql_error)?,
                "locator": serde_json::from_str::<Value>(&locator).map_err(db::json_error)?,
            });
            if let Some(chunk_role) = row
                .try_get::<Option<String>, _>("role")
                .map_err(db::sql_error)?
            {
                item["role"] = Value::String(chunk_role);
            }
            Ok(item)
        })
        .collect::<Result<Vec<Value>, String>>()?;

    let end = filter
        .offset
        .checked_add(chunks.len() as i64)
        .ok_or("node-chunks-invalid: offset overflow")?;
    Ok(NodeChunksSlice {
        total_chunks,
        next_offset: (end < total_chunks).then_some(end),
        roles,
        chunks,
    })
}

async fn load_role_counts(
    connection: &mut SqliteConnection,
    space_id: &str,
    node_id: &str,
) -> Result<Map<String, Value>, String> {
    let rows = sqlx::query_as::<_, (String, i64)>(
        "SELECT json_extract(c.metadata,'$.role') AS role, COUNT(*) \
         FROM spaces_current_knowledge_chunks c \
         JOIN spaces_semantic_nodes n ON n.id=c.node_id AND n.space_id=c.space_id \
         WHERE c.space_id=$1 AND c.node_id=$2 AND json_extract(c.metadata,'$.role') IS NOT NULL \
         GROUP BY role ORDER BY role",
    )
    .bind(space_id)
    .bind(node_id)
    .fetch_all(&mut *connection)
    .await
    .map_err(db::sql_error)?;
    Ok(rows
        .into_iter()
        .map(|(role, count)| (role, Value::from(count)))
        .collect())
}
