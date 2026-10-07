use lancedb::Connection;
use serde_json::Value;
use sqlx::{Row, SqlitePool};
use std::collections::HashMap;

use super::{
    current_chunks,
    generation::{IndexGeneration, REBUILD_REQUIRED},
    lancedb_store::open_knowledge_table_if_exists,
    predicates::escape_sql_literal,
    search::query_vectors,
};
use crate::spaces::db;

struct SearchSpace {
    id: String,
    title: String,
    generation: IndexGeneration,
}

pub(super) async fn search(
    pool: &SqlitePool,
    index: &Connection,
    query: &[f32],
    embedding_spec_id: &str,
    limit: i64,
) -> Result<Vec<Value>, String> {
    let spaces = active_spaces(pool, embedding_spec_id).await?;
    if spaces.is_empty() {
        return Ok(Vec::new());
    }
    let table = open_knowledge_table_if_exists(index)
        .await?
        .ok_or(REBUILD_REQUIRED)?;
    let generation_ids = spaces
        .iter()
        .map(|space| format!("'{}'", escape_sql_literal(&space.generation.id)))
        .collect::<Vec<_>>()
        .join(",");
    let filter = format!(
        "generation_id IN ({generation_ids}) AND embedding_spec_id='{}'",
        escape_sql_literal(embedding_spec_id)
    );
    let values = query_vectors(&table, query, filter, limit).await?;
    let current = active_spaces(pool, embedding_spec_id).await?;
    if !identities(&spaces).eq(identities(&current)) {
        return Err(REBUILD_REQUIRED.into());
    }
    let generations = current
        .iter()
        .map(|space| (space.id.as_str(), &space.generation))
        .collect::<Vec<_>>();
    let mut values = current_chunks::filter(pool, &generations, values).await?;
    let titles: HashMap<_, _> = current
        .iter()
        .map(|space| (space.id.as_str(), space.title.as_str()))
        .collect();
    for value in &mut values {
        let space_id = value["spaceId"]
            .as_str()
            .ok_or("Semantic search result has no Space identity")?;
        let title = titles
            .get(space_id)
            .ok_or("Semantic search returned an unrequested Space")?
            .to_string();
        value["spaceTitle"] = Value::from(title);
    }
    Ok(values)
}

fn identities(spaces: &[SearchSpace]) -> impl Iterator<Item = (&str, &str, &str)> {
    spaces.iter().map(|space| {
        (
            space.id.as_str(),
            space.generation.id.as_str(),
            space.generation.embedding_spec_id.as_str(),
        )
    })
}

async fn active_spaces(
    pool: &SqlitePool,
    embedding_spec_id: &str,
) -> Result<Vec<SearchSpace>, String> {
    let rows = sqlx::query(
        "SELECT s.id,s.title,g.id AS generation_id,g.embedding_spec_id FROM spaces s \
         LEFT JOIN spaces_retrieval_generations g ON g.space_id=s.id AND g.status='active' \
         WHERE s.archived_at IS NULL ORDER BY s.id",
    )
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    rows.into_iter()
        .map(|row| {
            let id: Option<String> = row.try_get("generation_id").map_err(db::sql_error)?;
            let spec: Option<String> = row.try_get("embedding_spec_id").map_err(db::sql_error)?;
            let generation = IndexGeneration {
                id: id.ok_or(REBUILD_REQUIRED)?,
                embedding_spec_id: spec
                    .filter(|spec| spec == embedding_spec_id)
                    .ok_or(REBUILD_REQUIRED)?,
            };
            Ok(SearchSpace {
                id: row.try_get("id").map_err(db::sql_error)?,
                title: row.try_get("title").map_err(db::sql_error)?,
                generation,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests;
