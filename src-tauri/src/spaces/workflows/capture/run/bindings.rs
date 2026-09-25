use std::collections::HashMap;
use std::collections::HashSet;

use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::{Row, Sqlite, Transaction};

use super::BoundResource;
use crate::spaces::{
    db, derivation::inputs::RevisionBasis, workflows::capture::types::CandidateInputSlot,
};

pub(super) async fn bind_inputs(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    slots: &[CandidateInputSlot],
    selected: &HashMap<String, Vec<String>>,
) -> Result<HashMap<String, Vec<BoundResource>>, String> {
    if selected.len() != slots.len() || space_id.trim().is_empty() {
        return Err("workflow-bindings-invalid: bind each input slot exactly once".into());
    }
    let mut bindings = HashMap::new();
    let mut used_nodes = HashSet::new();
    for slot in slots {
        let node_ids = selected
            .get(&slot.id)
            .ok_or("workflow-bindings-invalid: an input slot is unbound")?;
        if node_ids.len() < slot.min_items || node_ids.len() > slot.max_items {
            return Err("workflow-bindings-invalid: input count does not match the slot".into());
        }
        let mut resources = Vec::with_capacity(node_ids.len());
        for node_id in node_ids {
            if !used_nodes.insert(node_id) {
                return Err(
                    "workflow-bindings-invalid: bind distinct resources to distinct input slots"
                        .into(),
                );
            }
            let row = sqlx::query(
            "SELECT kind,data FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND deleted_at IS NULL",
        )
        .bind(node_id)
        .bind(space_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(db::sql_error)?
        .ok_or("workflow-bindings-invalid: selected resource is unavailable")?;
            let kind: String = row.try_get("kind").map_err(db::sql_error)?;
            if kind != slot.kind {
                return Err(
                "workflow-bindings-invalid: selected resource type does not match the input slot"
                    .into(),
            );
            }
            let data: String = row.try_get("data").map_err(db::sql_error)?;
            let value: Value = serde_json::from_str(&data).map_err(db::json_error)?;
            for basis in &slot.revision_bases {
                crate::spaces::derivation::input_revision::resolve(
                    &kind,
                    &value,
                    RevisionBasis::parse(basis)?,
                )
                .map_err(|error| format!("workflow-bindings-invalid: {error}"))?;
            }
            let data_digest = format!("{:x}", Sha256::digest(data.as_bytes()));
            resources.push(BoundResource {
                node_id: node_id.clone(),
                kind,
                data_digest,
            });
        }
        bindings.insert(slot.id.clone(), resources);
    }
    Ok(bindings)
}
