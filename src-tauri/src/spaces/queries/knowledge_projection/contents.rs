use serde::Serialize;
use serde_json::Value;
use sqlx::{Row, SqliteConnection};

use crate::spaces::db;

const SUMMARY_MAX_CHARS: usize = 160;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ContentsPage {
    pub space_id: String,
    pub title: String,
    pub total_nodes: i64,
    pub nodes: Vec<ContentNode>,
    pub next_offset: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ContentNode {
    pub node_id: String,
    pub kind: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_type_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<i64>,
    pub knowledge_revision: Option<String>,
    pub source_revision: Option<String>,
    pub indexed: bool,
    pub chunk_count: i64,
    pub updated_at: i64,
}

pub(crate) async fn list_contents(
    connection: &mut SqliteConnection,
    space_id: &str,
    offset: i64,
    limit: i64,
) -> Result<ContentsPage, String> {
    if offset < 0 || !(1..=100).contains(&limit) {
        return Err(
            "space-contents-invalid: offset must be nonnegative and limit must be 1–100".into(),
        );
    }
    let title: String =
        sqlx::query_scalar("SELECT title FROM spaces WHERE id=$1 AND archived_at IS NULL")
            .bind(space_id)
            .fetch_optional(&mut *connection)
            .await
            .map_err(db::sql_error)?
            .ok_or("space-unavailable: current Space does not exist or is archived")?;
    let total_nodes: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM spaces_nodes WHERE space_id=$1 AND deleted_at IS NULL \
         AND kind NOT IN ('credential','portal')",
    )
    .bind(space_id)
    .fetch_one(&mut *connection)
    .await
    .map_err(db::sql_error)?;
    let rows = sqlx::query(
        "SELECT n.id, n.kind, n.object_type_id, \
           COALESCE(json_extract(n.data,'$.title'), '') AS title, \
           json_extract(n.data,'$.status') AS status, \
           COALESCE( \
             NULLIF(trim(json_extract(n.data,'$.summary')), ''), \
             NULLIF(trim(json_extract(n.data,'$.mediaKnowledge.insights.summary')), ''), \
             NULLIF(trim(json_extract(n.data,'$.description')), ''), \
             NULLIF(trim(json_extract(n.data,'$.contentPreview')), ''), \
             NULLIF(trim(json_extract(n.data,'$.plainText')), ''), \
             NULLIF(trim(json_extract(n.data,'$.text')), '') \
           ) AS raw_summary, \
           json_extract(n.data,'$.tags') AS raw_tags, \
           CAST(json_extract(n.data,'$.durationSeconds') AS INTEGER) AS duration_seconds, \
           json_extract(n.data,'$.knowledgeRevision') AS knowledge_revision, \
           json_extract(n.data,'$.sourceRevision') AS source_revision, \
           (SELECT COUNT(*) FROM spaces_current_knowledge_chunks c \
            WHERE c.space_id=n.space_id AND c.node_id=n.id) AS chunk_count, \
           n.updated_at \
         FROM spaces_nodes n WHERE n.space_id=$1 AND n.deleted_at IS NULL \
         AND n.kind NOT IN ('credential','portal') ORDER BY n.id LIMIT $2 OFFSET $3",
    )
    .bind(space_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(&mut *connection)
    .await
    .map_err(db::sql_error)?;
    let nodes = rows
        .into_iter()
        .map(decode_content_node)
        .collect::<Result<Vec<_>, String>>()?;
    let end = offset
        .checked_add(nodes.len() as i64)
        .ok_or("space-contents-invalid: offset overflow")?;
    Ok(ContentsPage {
        space_id: space_id.to_owned(),
        title,
        total_nodes,
        nodes,
        next_offset: (end < total_nodes).then_some(end),
    })
}

fn decode_content_node(row: sqlx::sqlite::SqliteRow) -> Result<ContentNode, String> {
    let chunk_count: i64 = row.try_get("chunk_count").map_err(db::sql_error)?;
    let raw_summary: Option<String> = row.try_get("raw_summary").map_err(db::sql_error)?;
    let raw_tags: Option<String> = row.try_get("raw_tags").map_err(db::sql_error)?;
    Ok(ContentNode {
        node_id: row.try_get("id").map_err(db::sql_error)?,
        kind: row.try_get("kind").map_err(db::sql_error)?,
        title: row.try_get("title").map_err(db::sql_error)?,
        object_type_id: row.try_get("object_type_id").map_err(db::sql_error)?,
        status: row.try_get("status").map_err(db::sql_error)?,
        summary: raw_summary.and_then(|text| compact_summary(&text)),
        tags: raw_tags.and_then(|json| decode_tags(&json)),
        duration_seconds: row.try_get("duration_seconds").map_err(db::sql_error)?,
        knowledge_revision: row.try_get("knowledge_revision").map_err(db::sql_error)?,
        source_revision: row.try_get("source_revision").map_err(db::sql_error)?,
        indexed: chunk_count > 0,
        chunk_count,
        updated_at: row.try_get("updated_at").map_err(db::sql_error)?,
    })
}

fn compact_summary(raw: &str) -> Option<String> {
    let collapsed = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.is_empty() {
        return None;
    }
    let mut chars = collapsed.chars();
    let excerpt: String = chars.by_ref().take(SUMMARY_MAX_CHARS).collect();
    if chars.next().is_some() {
        Some(format!("{excerpt}…"))
    } else {
        Some(excerpt)
    }
}

fn decode_tags(raw: &str) -> Option<Vec<String>> {
    let Value::Array(items) = serde_json::from_str::<Value>(raw).ok()? else {
        return None;
    };
    let tags = items
        .into_iter()
        .filter_map(|item| item.as_str().map(str::trim).map(str::to_owned))
        .filter(|tag| !tag.is_empty())
        .collect::<Vec<_>>();
    (!tags.is_empty()).then_some(tags)
}
