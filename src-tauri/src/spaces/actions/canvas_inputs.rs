use std::collections::HashSet;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::{Sqlite, Transaction};

use super::canvas::SpaceActionSource;
use crate::spaces::{db, knowledge::indexing::revision};

pub(super) async fn pin(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    sources: &[SpaceActionSource],
) -> Result<Value, String> {
    let mut seen = HashSet::new();
    if sources.is_empty()
        || sources.len() > 32
        || sources.iter().any(|source| !seen.insert(&source.node_id))
    {
        return Err("action-input-invalid: select 1 to 32 distinct resources".into());
    }
    let mut inputs = Vec::with_capacity(sources.len());
    for (position, source) in sources.iter().enumerate() {
        let encoded: String = sqlx::query_scalar("SELECT data FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND deleted_at IS NULL AND kind NOT IN ('credential','portal','text')")
            .bind(&source.node_id).bind(space_id).fetch_optional(&mut **tx).await.map_err(db::sql_error)?
            .ok_or("action-input-unavailable: selected resource is unavailable or not authorized for derivation")?;
        let data: Value = serde_json::from_str(&encoded).map_err(db::json_error)?;
        revision::validate_declared(&data)?;
        if data.get("knowledgeRevision").and_then(Value::as_str) != Some(&source.knowledge_revision)
        {
            return Err("action-input-stale: selected resource changed before execution".into());
        }
        inputs.push(json!({"nodeId": source.node_id, "role": if position == 0 {"primary"} else {"source"}, "revisionBasis": "knowledge", "dataDigest": format!("{:x}", Sha256::digest(encoded.as_bytes()))}));
    }
    Ok(Value::Array(inputs))
}
