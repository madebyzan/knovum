use std::collections::HashMap;

use serde_json::Value;
use sqlx::{Row, Sqlite, Transaction};

use crate::spaces::db;

mod shape;

#[derive(Debug)]
struct CurrentCitationRef {
    chunk_id: String,
    node_id: String,
    revision: String,
}

pub async fn validate_conversation_citations(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    citations: &Value,
) -> Result<(), String> {
    let refs = shape::citation_shapes(citations)?;
    validate_live_refs(tx, space_id, refs).await
}

pub fn validate_citation_snapshots(citations: &Value) -> Result<(), String> {
    shape::citation_shapes(citations).map(|_| ())
}

async fn validate_live_refs(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    refs: Vec<CurrentCitationRef>,
) -> Result<(), String> {
    if refs.is_empty() {
        return Ok(());
    }
    let chunk_ids = refs
        .iter()
        .map(|citation| citation.chunk_id.as_str())
        .collect::<Vec<_>>();
    let rows = sqlx::query(
        "SELECT c.id,c.node_id,c.source_revision FROM spaces_current_knowledge_chunks c \
         WHERE c.space_id=$1 AND c.id IN (SELECT value FROM json_each($2))",
    )
    .bind(space_id)
    .bind(sqlx::types::Json(&chunk_ids))
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let mut stored = HashMap::with_capacity(rows.len());
    for row in rows {
        stored.insert(
            row.try_get::<String, _>("id").map_err(db::sql_error)?,
            (
                row.try_get::<String, _>("node_id").map_err(db::sql_error)?,
                row.try_get::<String, _>("source_revision")
                    .map_err(db::sql_error)?,
            ),
        );
    }
    for citation in refs {
        let Some((node_id, revision)) = stored.get(&citation.chunk_id) else {
            return Err("Grounded AI citation chunk is unavailable or stale".into());
        };
        if node_id != &citation.node_id {
            return Err("Grounded AI citation chunk does not belong to its semantic node".into());
        }
        if revision != &citation.revision {
            return Err("Grounded AI citation source revision does not match its chunk".into());
        }
    }
    Ok(())
}
