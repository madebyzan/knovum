use std::collections::HashMap;

use sqlx::{Row, Sqlite, Transaction};

use crate::spaces::db;

#[derive(Debug, Clone)]
pub(in crate::spaces) struct NodeChunkRevisions {
    knowledge: Option<String>,
    source: Option<String>,
}

impl NodeChunkRevisions {
    pub(in crate::spaces) fn expected(&self, origin: &str) -> Result<&str, String> {
        match origin {
            "derived" => self.knowledge.as_deref(),
            "source" | "media" => self.source.as_deref(),
            _ => {
                return Err(
                    "Only supplemental knowledge origins can be restored explicitly".into(),
                );
            }
        }
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("Knowledge {origin} chunks require the owning node revision"))
    }
}

pub(in crate::spaces) async fn load_revision_contracts(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    node_ids: &[String],
) -> Result<HashMap<String, NodeChunkRevisions>, String> {
    if node_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let ids = serde_json::to_string(node_ids).map_err(db::json_error)?;
    let rows = sqlx::query(
        "SELECT id,json_extract(data,'$.knowledgeRevision') AS knowledge_revision,\
         json_extract(data,'$.sourceRevision') AS source_revision FROM spaces_nodes \
         WHERE space_id=$1 AND deleted_at IS NULL AND id IN (SELECT value FROM json_each($2))",
    )
    .bind(space_id)
    .bind(ids)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let mut contracts = HashMap::with_capacity(rows.len());
    for row in rows {
        contracts.insert(
            row.try_get("id").map_err(db::sql_error)?,
            NodeChunkRevisions {
                knowledge: row.try_get("knowledge_revision").map_err(db::sql_error)?,
                source: row.try_get("source_revision").map_err(db::sql_error)?,
            },
        );
    }
    Ok(contracts)
}
