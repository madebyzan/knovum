use std::collections::HashMap;

use arrow_array::RecordBatch;
use futures::TryStreamExt;
use lancedb::query::{ExecutableQuery, QueryBase};
use serde::Serialize;
use serde_json::Value;
use sqlx::Row;
use tauri::{AppHandle, Runtime};

use crate::{ai::capabilities::embedding, spaces::db};

use super::{
    lancedb_store::open_knowledge_table, predicates::escape_sql_literal,
    search::search_semantic_vector,
};

mod source;
use source::{centroid, source_vectors};

const SOURCE_LIMIT: usize = 4;
const CANDIDATE_LIMIT: i64 = 120;
const RESULT_LIMIT: usize = 12;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelatedContentResponse {
    state: &'static str,
    items: Vec<RelatedContentItem>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RelatedContentItem {
    node_id: String,
    node_kind: String,
    node_title: String,
    snippet: String,
    score: f64,
}

pub async fn list<R: Runtime>(
    app: &AppHandle<R>,
    space_id: &str,
    node_id: &str,
) -> Result<RelatedContentResponse, String> {
    if space_id.is_empty() || node_id.is_empty() {
        return Err("Space and content identifiers are required".to_owned());
    }
    let pool = db::pool(app).await?;
    let available: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM spaces_current_knowledge_chunks c \
         JOIN spaces_nodes n ON n.id=c.node_id AND n.space_id=c.space_id \
         WHERE c.space_id=$1 AND c.node_id=$2 AND n.deleted_at IS NULL",
    )
    .bind(space_id)
    .bind(node_id)
    .fetch_one(pool)
    .await
    .map_err(db::sql_error)?;
    if available == 0 {
        return Ok(RelatedContentResponse {
            state: "no_content",
            items: Vec::new(),
        });
    }

    let rows = sqlx::query(
        "WITH indexed AS ( \
           SELECT c.id, c.source_revision, \
                  ROW_NUMBER() OVER (ORDER BY c.created_at, c.id) AS position, \
                  COUNT(*) OVER () AS total \
           FROM spaces_current_knowledge_chunks c \
           JOIN spaces_chunk_embeddings e ON e.chunk_id=c.id AND e.source_revision=c.source_revision \
           WHERE c.space_id=$1 AND c.node_id=$2 AND e.model=$3 \
         ) \
         SELECT id, source_revision FROM indexed \
         WHERE position IN (1, (total+2)/3, (2*total+2)/3, total) \
         ORDER BY position LIMIT $4",
    )
    .bind(space_id)
    .bind(node_id)
    .bind(embedding::active_text_provider().model_id())
    .bind(SOURCE_LIMIT as i64)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    if rows.is_empty() {
        return Ok(RelatedContentResponse {
            state: "unindexed",
            items: Vec::new(),
        });
    }
    let chunk_revisions = rows
        .iter()
        .map(|row| {
            Ok((
                row.try_get::<String, _>("id").map_err(db::sql_error)?,
                row.try_get::<String, _>("source_revision")
                    .map_err(db::sql_error)?,
            ))
        })
        .collect::<Result<HashMap<_, _>, String>>()?;
    let table = open_knowledge_table(app).await?;
    let id_filter = chunk_revisions
        .keys()
        .map(|id| format!("'{}'", escape_sql_literal(id)))
        .collect::<Vec<_>>()
        .join(",");
    let filter = format!(
        "space_id = '{}' AND node_id = '{}' AND model = '{}' AND chunk_id IN ({id_filter})",
        escape_sql_literal(space_id),
        escape_sql_literal(node_id),
        escape_sql_literal(embedding::active_text_provider().model_id()),
    );
    let batches = table
        .query()
        .only_if(filter)
        .limit(SOURCE_LIMIT)
        .execute()
        .await
        .map_err(|error| format!("Failed to read source embeddings: {error}"))?
        .try_collect::<Vec<RecordBatch>>()
        .await
        .map_err(|error| format!("Failed to collect source embeddings: {error}"))?;
    let vectors = source_vectors(&batches, &chunk_revisions)?;
    if vectors.is_empty() {
        return Err("Current source embeddings are missing from the retrieval index".to_owned());
    }

    let query = centroid(&vectors)?;
    let mut by_node: HashMap<String, RelatedContentItem> = HashMap::new();
    for match_value in
        search_semantic_vector(app, space_id, &query, None, Some(node_id), CANDIDATE_LIMIT).await?
    {
        collect_match(&mut by_node, &match_value, node_id)?;
    }
    let mut items = by_node.into_values().collect::<Vec<_>>();
    items.sort_by(|left, right| right.score.total_cmp(&left.score));
    items.truncate(RESULT_LIMIT);
    Ok(RelatedContentResponse {
        state: "ready",
        items,
    })
}

fn collect_match(
    by_node: &mut HashMap<String, RelatedContentItem>,
    value: &Value,
    source_node_id: &str,
) -> Result<(), String> {
    let field = |name: &str| {
        value
            .get(name)
            .and_then(Value::as_str)
            .ok_or_else(|| format!("Related content result is missing {name}"))
    };
    let node_id = field("nodeId")?;
    if node_id == source_node_id {
        return Ok(());
    }
    let score = value
        .get("score")
        .and_then(Value::as_f64)
        .ok_or_else(|| "Related content result is missing score".to_owned())?;
    let current = by_node.get(node_id);
    if current.is_some_and(|item| item.score >= score) {
        return Ok(());
    }
    by_node.insert(
        node_id.to_owned(),
        RelatedContentItem {
            node_id: node_id.to_owned(),
            node_kind: field("nodeKind")?.to_owned(),
            node_title: field("nodeTitle")?.to_owned(),
            snippet: field("snippet")?.to_owned(),
            score,
        },
    );
    Ok(())
}
