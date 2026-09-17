use serde::Serialize;
use sqlx::Row;

use crate::spaces::db;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DerivationSource {
    node_id: Option<String>,
    source_identity: String,
    title: String,
    role: String,
    source_revision: String,
    current: bool,
}

pub async fn load(
    pool: &sqlx::SqlitePool,
    derivation_id: &str,
) -> Result<Vec<DerivationSource>, String> {
    let rows = sqlx::query("SELECT i.source_identity,i.node_id,i.role,i.source_revision,json_extract(i.source_snapshot,'$.title') AS title,COALESCE(json_extract(n.data,'$.knowledgeRevision'),json_extract(n.data,'$.sourceRevision')) AS current_revision,n.deleted_at,n.space_id=d.space_id AS same_space FROM spaces_knowledge_derivation_inputs i JOIN spaces_knowledge_derivations d ON d.id=i.derivation_id LEFT JOIN spaces_nodes n ON n.id=i.node_id WHERE i.derivation_id=$1 ORDER BY i.position")
        .bind(derivation_id)
        .fetch_all(pool)
        .await
        .map_err(db::sql_error)?;
    rows.into_iter()
        .map(|row| {
            let source_revision: String = row.try_get("source_revision").map_err(db::sql_error)?;
            let current_revision: Option<String> =
                row.try_get("current_revision").map_err(db::sql_error)?;
            let node_id: Option<String> = row.try_get("node_id").map_err(db::sql_error)?;
            let deleted_at: Option<i64> = row.try_get("deleted_at").map_err(db::sql_error)?;
            let same_space: Option<bool> = row.try_get("same_space").map_err(db::sql_error)?;
            let available = node_id.is_some() && deleted_at.is_none() && same_space == Some(true);
            Ok(DerivationSource {
                node_id: if available { node_id } else { None },
                source_identity: row.try_get("source_identity").map_err(db::sql_error)?,
                title: row.try_get("title").map_err(db::sql_error)?,
                role: row.try_get("role").map_err(db::sql_error)?,
                current: available && current_revision.as_deref() == Some(source_revision.as_str()),
                source_revision,
            })
        })
        .collect()
}
