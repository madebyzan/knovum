use std::collections::{hash_map::Entry, HashMap};

use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::{sqlite::SqliteRow, QueryBuilder, Row, Sqlite, SqliteConnection};
use tauri::AppHandle;

use super::excerpt::build_excerpt;
use crate::{
    ai::capabilities::embedding,
    spaces::{
        db,
        knowledge::{retrieval, search_plan},
    },
};

const MAX_EXCERPT_CHARS: usize = 280;

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SearchOutputMode {
    /// Returns individual matched passages with excerpts, locators, and cite ids.
    #[default]
    Passages,
    /// Aggregates hits per node (hit count + top excerpt) for low-token Space-wide surveys.
    Nodes,
}

pub(crate) struct SearchFilter {
    pub(crate) query: String,
    pub(crate) limit: i64,
    pub(crate) node_ids: Option<Vec<String>>,
    pub(crate) roles: Option<Vec<String>>,
    pub(crate) mode: SearchOutputMode,
}

pub(crate) async fn search_knowledge(
    connection: &mut SqliteConnection,
    space_id: &str,
    filter: &SearchFilter,
) -> Result<Value, String> {
    let limit = filter.limit.clamp(1, 100);
    let candidate_limit = candidate_limit(limit, filter.mode);
    let passages = search_lexical_passages(connection, space_id, filter, candidate_limit).await?;
    Ok(finalize_search_output(
        passages,
        limit as usize,
        filter.mode,
    ))
}

/// Runs lexical search and, when an active semantic index and local embedding model are already
/// ready for `space_id`, fuses semantic matches via reciprocal rank fusion.
pub(crate) async fn search_knowledge_hybrid(
    app: &AppHandle,
    space_id: &str,
    filter: &SearchFilter,
) -> Result<Value, String> {
    let limit = filter.limit.clamp(1, 100);
    let candidate_limit = candidate_limit(limit, filter.mode);
    let plan = search_plan::build(filter.query.trim())?;
    let pool = db::pool(app).await?;
    let mut reader = pool.acquire().await.map_err(db::sql_error)?;
    let lexical = search_lexical_passages(&mut reader, space_id, filter, candidate_limit).await?;
    drop(reader);

    let active_generation =
        retrieval::generation::find_active(pool, space_id, embedding::EMBEDDING_SPEC_ID).await?;
    if active_generation.is_none() {
        return Ok(finalize_search_output(lexical, limit as usize, filter.mode));
    }
    let cancel = tokio_util::sync::CancellationToken::new();
    let vector = match retrieval::search::embed_query(app, filter.query.trim(), space_id, &cancel)
        .await
    {
        Ok(vector) => vector,
        Err(error) => {
            log::warn!(
                    "MCP hybrid search query embedding failed for space {space_id}, returning lexical matches: {error}"
                );
            return Ok(finalize_search_output(lexical, limit as usize, filter.mode));
        }
    };
    let node_ids = non_empty(filter.node_ids.as_deref());
    let semantic_raw = match retrieval::search::search_semantic_vector(
        app,
        space_id,
        &vector,
        node_ids,
        None,
        candidate_limit,
    )
    .await
    {
        Ok(matches) => matches,
        Err(error) => {
            log::warn!(
                "MCP hybrid search vector query failed for space {space_id}, returning lexical matches: {error}"
            );
            return Ok(finalize_search_output(lexical, limit as usize, filter.mode));
        }
    };
    let roles = non_empty(filter.roles.as_deref());
    let semantic = semantic_raw
        .into_iter()
        .filter(|item| matches_role(item, roles))
        .map(|item| project_semantic_match(item, &plan.terms))
        .collect::<Vec<_>>();
    let fused = if semantic.is_empty() {
        lexical
    } else {
        fuse_matches(lexical, semantic, candidate_limit as usize)
    };
    Ok(finalize_search_output(fused, limit as usize, filter.mode))
}

async fn search_lexical_passages(
    connection: &mut SqliteConnection,
    space_id: &str,
    filter: &SearchFilter,
    limit: i64,
) -> Result<Vec<Value>, String> {
    let trimmed = filter.query.trim();
    if trimmed.is_empty() {
        return Err("invalid-query: query must be non-empty".into());
    }
    let plan = search_plan::build(trimmed)?;
    if plan.branches.is_empty() {
        return Err("invalid-query: query must contain searchable text".into());
    }
    let node_ids = non_empty(filter.node_ids.as_deref());
    let roles = non_empty(filter.roles.as_deref());
    if let Some(fts_query) = plan.fts_query {
        let rows = search_fts(connection, space_id, &fts_query, node_ids, roles, limit).await?;
        rows.into_iter()
            .map(|row| decode_search_projection(row, None))
            .collect()
    } else {
        let rows =
            search_substring(connection, space_id, &plan.branches, node_ids, roles, limit).await?;
        rows.into_iter()
            .map(|row| decode_search_projection(row, Some(&plan.terms)))
            .collect()
    }
}

fn candidate_limit(limit: i64, mode: SearchOutputMode) -> i64 {
    match mode {
        SearchOutputMode::Passages => limit,
        SearchOutputMode::Nodes => (limit.saturating_mul(4)).clamp(40, 200),
    }
}

fn finalize_search_output(passages: Vec<Value>, limit: usize, mode: SearchOutputMode) -> Value {
    match mode {
        SearchOutputMode::Passages => Value::Array(passages.into_iter().take(limit).collect()),
        SearchOutputMode::Nodes => Value::Array(aggregate_by_node(passages, limit)),
    }
}

fn aggregate_by_node(passages: Vec<Value>, limit: usize) -> Vec<Value> {
    let mut order: Vec<Value> = Vec::new();
    let mut index_by_node_id: HashMap<String, usize> = HashMap::new();
    for passage in passages {
        let Some(node_id) = passage
            .get("nodeId")
            .and_then(Value::as_str)
            .map(str::to_owned)
        else {
            continue;
        };
        if let Some(&slot) = index_by_node_id.get(&node_id) {
            if let Some(count) = order[slot]["matchCount"].as_u64() {
                order[slot]["matchCount"] = Value::from(count + 1);
            }
            continue;
        }
        let mut entry = json!({
            "nodeId": node_id,
            "kind": passage["kind"],
            "title": passage["title"],
            "matchCount": 1u64,
            "topChunkId": passage["chunkId"],
            "topPreview": passage["preview"],
            "topLocator": passage["locator"],
        });
        if let Some(role) = passage.get("role").filter(|value| !value.is_null()) {
            entry["topRole"] = role.clone();
        }
        index_by_node_id.insert(node_id, order.len());
        order.push(entry);
    }
    order.into_iter().take(limit).collect()
}

async fn search_fts(
    connection: &mut SqliteConnection,
    space_id: &str,
    fts_query: &str,
    node_ids: Option<&[String]>,
    roles: Option<&[String]>,
    limit: i64,
) -> Result<Vec<SqliteRow>, String> {
    let mut sql = QueryBuilder::<Sqlite>::new(
        "SELECT spaces_knowledge_fts.node_id AS id, n.kind, \
           json_extract(n.data,'$.title') AS title, \
           snippet(spaces_knowledge_fts,3,'‹','›',' … ',28) AS preview, \
           bm25(spaces_knowledge_fts) AS rank, c.id AS chunk_id, c.source_revision, \
           c.locator, json_extract(c.metadata,'$.role') AS role, n.updated_at \
         FROM spaces_knowledge_fts \
         JOIN spaces_current_knowledge_chunks c ON c.id=spaces_knowledge_fts.chunk_id \
         JOIN spaces_semantic_nodes n ON n.id=c.node_id AND n.space_id=c.space_id \
         WHERE spaces_knowledge_fts.space_id=",
    );
    sql.push_bind(space_id.to_owned())
        .push(" AND spaces_knowledge_fts MATCH ")
        .push_bind(fts_query.to_owned());
    push_scope_filters(&mut sql, node_ids, roles);
    sql.push(" ORDER BY rank LIMIT ").push_bind(limit);
    sql.build()
        .fetch_all(&mut *connection)
        .await
        .map_err(db::sql_error)
}

async fn search_substring(
    connection: &mut SqliteConnection,
    space_id: &str,
    branches: &[Vec<String>],
    node_ids: Option<&[String]>,
    roles: Option<&[String]>,
    limit: i64,
) -> Result<Vec<SqliteRow>, String> {
    let mut sql = QueryBuilder::<Sqlite>::new(
        "SELECT c.node_id AS id, n.kind, json_extract(n.data,'$.title') AS title, \
           c.text AS preview, -1.0 AS rank, c.id AS chunk_id, c.source_revision, \
           c.locator, json_extract(c.metadata,'$.role') AS role, n.updated_at \
         FROM spaces_current_knowledge_chunks c \
         JOIN spaces_semantic_nodes n ON n.id=c.node_id AND n.space_id=c.space_id \
         WHERE c.space_id=",
    );
    sql.push_bind(space_id.to_owned());
    sql.push(" AND (");
    for (branch_index, branch_terms) in branches.iter().enumerate() {
        if branch_index > 0 {
            sql.push(" OR ");
        }
        sql.push("(");
        for (term_index, term) in branch_terms.iter().enumerate() {
            if term_index > 0 {
                sql.push(" AND ");
            }
            sql.push("instr(lower(c.text), lower(")
                .push_bind(term.clone())
                .push(")) > 0");
        }
        sql.push(")");
    }
    sql.push(")");
    push_scope_filters(&mut sql, node_ids, roles);
    sql.push(" ORDER BY c.updated_at DESC LIMIT ")
        .push_bind(limit);
    sql.build()
        .fetch_all(&mut *connection)
        .await
        .map_err(db::sql_error)
}

fn push_scope_filters(
    sql: &mut QueryBuilder<Sqlite>,
    node_ids: Option<&[String]>,
    roles: Option<&[String]>,
) {
    if let Some(ids) = node_ids {
        sql.push(" AND c.node_id IN (SELECT value FROM json_each(")
            .push_bind(sqlx::types::Json(ids.to_vec()))
            .push("))");
    }
    if let Some(roles) = roles {
        sql.push(" AND json_extract(c.metadata,'$.role') IN (SELECT value FROM json_each(")
            .push_bind(sqlx::types::Json(roles.to_vec()))
            .push("))");
    }
}

fn decode_search_projection(
    row: SqliteRow,
    excerpt_terms: Option<&[String]>,
) -> Result<Value, String> {
    let raw_preview: String = row.try_get("preview").map_err(db::sql_error)?;
    let preview = match excerpt_terms {
        Some(terms) => build_excerpt(&raw_preview, terms, MAX_EXCERPT_CHARS),
        None => raw_preview,
    };
    let mut item = json!({
        "nodeId": row.try_get::<String, _>("id").map_err(db::sql_error)?,
        "kind": row.try_get::<String, _>("kind").map_err(db::sql_error)?,
        "title": row.try_get::<Option<String>, _>("title").map_err(db::sql_error)?,
        "preview": preview,
        "rank": row.try_get::<f64, _>("rank").map_err(db::sql_error)?,
        "updatedAt": row.try_get::<i64, _>("updated_at").map_err(db::sql_error)?,
        "chunkId": row.try_get::<String, _>("chunk_id").map_err(db::sql_error)?,
        "sourceRevision": row.try_get::<String, _>("source_revision").map_err(db::sql_error)?,
        "locator": serde_json::from_str::<Value>(&row.try_get::<String, _>("locator").map_err(db::sql_error)?).map_err(db::json_error)?,
    });
    if let Some(role) = row
        .try_get::<Option<String>, _>("role")
        .map_err(db::sql_error)?
    {
        item["role"] = Value::String(role);
    }
    Ok(item)
}

fn matches_role(item: &Value, roles: Option<&[String]>) -> bool {
    let Some(roles) = roles else {
        return true;
    };
    item.get("metadata")
        .and_then(|meta| meta.get("role"))
        .and_then(Value::as_str)
        .is_some_and(|role| roles.iter().any(|allowed| allowed == role))
}

fn project_semantic_match(item: Value, terms: &[String]) -> Value {
    let preview = item
        .get("text")
        .and_then(Value::as_str)
        .map(|text| build_excerpt(text, terms, MAX_EXCERPT_CHARS))
        .unwrap_or_default();
    let mut projected = json!({
        "nodeId": item["nodeId"],
        "kind": item["nodeKind"],
        "title": item["nodeTitle"],
        "preview": preview,
        "rank": item["score"],
        "chunkId": item["chunkId"],
        "sourceRevision": item["sourceRevision"],
        "locator": item["locator"],
    });
    if let Some(role) = item
        .get("metadata")
        .and_then(|meta| meta.get("role"))
        .and_then(Value::as_str)
    {
        projected["role"] = Value::String(role.to_owned());
    }
    projected
}

fn fuse_matches(lexical: Vec<Value>, semantic: Vec<Value>, limit: usize) -> Vec<Value> {
    let mut ranked: HashMap<String, (Value, f64)> = HashMap::new();
    for (index, item) in lexical.into_iter().enumerate() {
        insert_ranked(&mut ranked, item, retrieval::hybrid::rrf_rank_score(index));
    }
    for (index, item) in semantic.into_iter().enumerate() {
        insert_ranked(&mut ranked, item, retrieval::hybrid::rrf_rank_score(index));
    }
    let mut values = ranked.into_values().collect::<Vec<_>>();
    values.sort_by(|left, right| right.1.total_cmp(&left.1));
    values
        .into_iter()
        .take(limit)
        .map(|(item, _)| item)
        .collect()
}

fn insert_ranked(ranked: &mut HashMap<String, (Value, f64)>, item: Value, score: f64) {
    let Some(chunk_id) = item
        .get("chunkId")
        .and_then(Value::as_str)
        .map(str::to_owned)
    else {
        return;
    };
    match ranked.entry(chunk_id) {
        Entry::Occupied(mut entry) => entry.get_mut().1 += score,
        Entry::Vacant(entry) => {
            entry.insert((item, score));
        }
    }
}

fn non_empty(items: Option<&[String]>) -> Option<&[String]> {
    items.filter(|slice| !slice.is_empty())
}
