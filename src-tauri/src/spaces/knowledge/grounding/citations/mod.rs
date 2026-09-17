use std::collections::{HashMap, HashSet};

use serde_json::Value;
use sqlx::{Row, Sqlite, Transaction};

use super::{db, GroundedThreadContract};

mod shape;

#[derive(Debug)]
struct CurrentCitationRef {
    chunk_id: String,
    node_id: String,
    revision: String,
}

pub async fn validate_current_citations(
    tx: &mut Transaction<'_, Sqlite>,
    contract: &GroundedThreadContract,
    citations: &Value,
) -> Result<(), String> {
    let refs = shape::citation_shapes(citations, contract.scope == "all-spaces")?;
    validate_live_refs(tx, contract, refs).await
}

pub async fn validate_historical_citations(
    tx: &mut Transaction<'_, Sqlite>,
    contract: &GroundedThreadContract,
    citations: &Value,
) -> Result<(), String> {
    let refs = shape::citation_shapes(citations, true)?;
    validate_live_refs(tx, contract, refs).await
}

async fn validate_live_refs(
    tx: &mut Transaction<'_, Sqlite>,
    contract: &GroundedThreadContract,
    refs: Vec<CurrentCitationRef>,
) -> Result<(), String> {
    if refs.is_empty() {
        return Ok(());
    }
    let allowed_nodes: HashSet<&str> = contract.node_ids.iter().map(String::as_str).collect();
    if !matches!(contract.scope.as_str(), "space" | "all-spaces")
        && refs
            .iter()
            .any(|citation| !allowed_nodes.contains(citation.node_id.as_str()))
    {
        return Err("Grounded AI citation is outside the persisted thread scope".into());
    }
    let chunk_ids = refs
        .iter()
        .map(|citation| citation.chunk_id.as_str())
        .collect::<Vec<_>>();
    let chunk_ids_json = serde_json::to_string(&chunk_ids).map_err(db::json_error)?;
    let rows = sqlx::query(
        "SELECT c.id,c.node_id,c.source_revision FROM spaces_current_knowledge_chunks c \
         WHERE c.space_id=$1 AND c.id IN (SELECT value FROM json_each($2))",
    )
    .bind(&contract.space_id)
    .bind(chunk_ids_json)
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
