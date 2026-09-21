use std::collections::{hash_map::Entry, HashMap};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use tauri::{AppHandle, Runtime};

use crate::spaces::{db, queries::knowledge};

use super::search::{embed_query, search_semantic_vector};

const MATCH_LIMIT: usize = 80;
const SPACE_SEMANTIC_LIMIT: i64 = 40;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HybridSearchRequest {
    pub space_id: Option<String>,
    pub query: String,
    pub node_ids: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HybridSearchResponse {
    pub matches: Vec<Value>,
    pub semantic: &'static str,
}

pub async fn search<R: Runtime>(
    app: &AppHandle<R>,
    request: HybridSearchRequest,
) -> Result<HybridSearchResponse, String> {
    let query = request.query.trim();
    if query.is_empty() {
        return Ok(HybridSearchResponse {
            matches: Vec::new(),
            semantic: "empty",
        });
    }

    let (vector_result, lexical) = match request.space_id.as_deref() {
        Some(space_id) => {
            let lex =
                knowledge::search_space(app, space_id, query, request.node_ids.clone()).await?;
            (embed_query(app, query).await, lex)
        }
        None => {
            let lex = knowledge::search_global(app, query).await?;
            (embed_query(app, query).await, lex)
        }
    };

    let (semantic, semantic_state) = match vector_result {
        Ok(vector) => {
            let sem = match request.space_id.as_deref() {
                Some(space_id) => search_semantic_vector(
                    app,
                    space_id,
                    &vector,
                    request.node_ids.as_deref(),
                    MATCH_LIMIT as i64,
                )
                .await
                .unwrap_or_default(),
                None => search_all_spaces(app, &vector).await.unwrap_or_default(),
            };
            let state = if sem.is_empty() { "empty" } else { "ready" };
            (sem, state)
        }
        Err(error) => {
            log::info!("semantic vector embedding unavailable: {error}");
            (Vec::new(), "unavailable")
        }
    };

    Ok(HybridSearchResponse {
        matches: reciprocal_rank_fusion(lexical, semantic),
        semantic: semantic_state,
    })
}

async fn search_all_spaces<R: Runtime>(
    app: &AppHandle<R>,
    vector: &[f32],
) -> Result<Vec<Value>, String> {
    let pool = db::pool(app).await?;
    let rows = sqlx::query(
        "SELECT id,title FROM spaces WHERE archived_at IS NULL ORDER BY updated_at DESC",
    )
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    let mut matches = Vec::new();
    for row in rows {
        let space_id: String = row.try_get("id").map_err(db::sql_error)?;
        let space_title: String = row.try_get("title").map_err(db::sql_error)?;
        let mut space_matches = match search_semantic_vector(
            app,
            &space_id,
            vector,
            None,
            SPACE_SEMANTIC_LIMIT,
        )
        .await
        {
            Ok(found) => found,
            Err(error) => {
                log::warn!("semantic vector search failed for space {space_id}: {error}");
                continue;
            }
        };
        for value in &mut space_matches {
            let object = value
                .as_object_mut()
                .ok_or_else(|| "Semantic search result must be an object".to_owned())?;
            object.insert("spaceId".to_owned(), Value::from(space_id.clone()));
            object.insert("spaceTitle".to_owned(), Value::from(space_title.clone()));
        }
        matches.extend(space_matches);
    }
    Ok(matches)
}

pub(super) fn reciprocal_rank_fusion(lexical: Vec<Value>, semantic: Vec<Value>) -> Vec<Value> {
    let mut ranked: HashMap<String, (Value, f64)> = HashMap::new();
    add_ranked(&mut ranked, lexical, "lexicalScore");
    add_ranked(&mut ranked, semantic, "semanticScore");
    let mut values = ranked.into_values().collect::<Vec<_>>();
    values.sort_by(|left, right| right.1.total_cmp(&left.1));
    values
        .into_iter()
        .take(MATCH_LIMIT)
        .map(|(mut value, score)| {
            if let Some(object) = value.as_object_mut() {
                object.insert("score".to_owned(), Value::from(score));
            }
            value
        })
        .collect()
}

fn add_ranked(target: &mut HashMap<String, (Value, f64)>, values: Vec<Value>, score_field: &str) {
    for (index, mut value) in values.into_iter().enumerate() {
        let Some(chunk_id) = value
            .get("chunkId")
            .and_then(Value::as_str)
            .map(str::to_owned)
        else {
            continue;
        };
        let Some(original_score) = value.get("score").and_then(Value::as_f64) else {
            log::warn!("retrieval result for chunk {chunk_id} is missing a valid score");
            continue;
        };
        let rank_score = 1.0 / (61.0 + index as f64);
        match target.entry(chunk_id) {
            Entry::Occupied(mut entry) => {
                let (stored, total) = entry.get_mut();
                *total += rank_score;
                insert_score(stored, score_field, original_score);
            }
            Entry::Vacant(entry) => {
                insert_score(&mut value, score_field, original_score);
                entry.insert((value, rank_score));
            }
        }
    }
}

fn insert_score(target: &mut Value, score_field: &str, score: f64) {
    if let Some(object) = target.as_object_mut() {
        object.insert(score_field.to_owned(), Value::from(score));
    }
}
