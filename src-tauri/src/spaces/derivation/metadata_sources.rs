use serde::Serialize;
use serde_json::Value;
use sqlx::Row;

use super::input_revision::{self, RevisionBasis};
use crate::spaces::db;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DerivationSource {
    node_id: Option<String>,
    source_identity: String,
    title: String,
    role: String,
    source_revision: String,
    revision_basis: String,
    input_revision: String,
    current: bool,
}

pub async fn load(
    pool: &sqlx::SqlitePool,
    derivation_id: &str,
) -> Result<Vec<DerivationSource>, String> {
    let rows = sqlx::query("SELECT i.source_identity,i.node_id,i.role,i.source_revision,i.revision_basis,i.input_revision,json_extract(i.source_snapshot,'$.title') AS title,n.kind,n.data,n.deleted_at,n.space_id=d.space_id AS same_space FROM spaces_knowledge_derivation_inputs i JOIN spaces_knowledge_derivations d ON d.id=i.derivation_id LEFT JOIN spaces_nodes n ON n.id=i.node_id WHERE i.derivation_id=$1 ORDER BY i.position")
        .bind(derivation_id)
        .fetch_all(pool)
        .await
        .map_err(db::sql_error)?;
    rows.into_iter()
        .map(|row| {
            let source_revision: String = row.try_get("source_revision").map_err(db::sql_error)?;
            let revision_basis: String = row.try_get("revision_basis").map_err(db::sql_error)?;
            let input_revision: String = row.try_get("input_revision").map_err(db::sql_error)?;
            let node_id: Option<String> = row.try_get("node_id").map_err(db::sql_error)?;
            let deleted_at: Option<i64> = row.try_get("deleted_at").map_err(db::sql_error)?;
            let same_space: Option<bool> = row.try_get("same_space").map_err(db::sql_error)?;
            let available = node_id.is_some() && deleted_at.is_none() && same_space == Some(true);
            let current = if available {
                let kind: String = row.try_get("kind").map_err(db::sql_error)?;
                let encoded: String = row.try_get("data").map_err(db::sql_error)?;
                let data: Value = serde_json::from_str(&encoded).map_err(db::json_error)?;
                let basis = RevisionBasis::parse(&revision_basis)?;
                input_revision::resolve(&kind, &data, basis)
                    .is_ok_and(|(revision, _)| revision == input_revision)
            } else {
                false
            };
            Ok(DerivationSource {
                node_id: if available { node_id } else { None },
                source_identity: row.try_get("source_identity").map_err(db::sql_error)?,
                title: row.try_get("title").map_err(db::sql_error)?,
                role: row.try_get("role").map_err(db::sql_error)?,
                current,
                source_revision,
                revision_basis,
                input_revision,
            })
        })
        .collect()
}
