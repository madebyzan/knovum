mod cleanup;

use std::collections::{BTreeSet, HashMap, HashSet};

use serde::{Deserialize, Serialize};
use sqlx::Row;
use tauri::{AppHandle, Runtime};

use crate::spaces::{db, structure::StructureNode};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct PermanentlyDeleteStructureRequest {
    pub(crate) space_id: String,
    pub(crate) batch_id: String,
    pub(crate) root_structure_node_id: Option<String>,
}

pub(crate) async fn permanently_delete<R: Runtime>(
    app: &AppHandle<R>,
    request: PermanentlyDeleteStructureRequest,
) -> Result<Vec<String>, String> {
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    let unit = load_unit(&mut tx, &request).await?;
    let structure_node_ids: HashSet<&str> = unit.structure.iter().map(StructureNode::id).collect();
    super::super::portal_integrity::reject_structure_targets(
        &mut tx,
        &request.space_id,
        &structure_node_ids,
    )
    .await?;
    let node_ids: HashSet<String> = unit
        .structure
        .iter()
        .filter_map(|structure_node| match structure_node {
            StructureNode::Content { node_id, .. } => Some(node_id.clone()),
            _ => None,
        })
        .collect();
    let node_refs: HashSet<&str> = node_ids.iter().map(String::as_str).collect();
    super::super::portal_integrity::reject_node_targets(&mut tx, &request.space_id, &node_refs)
        .await?;
    cleanup::reject_object_dependents(&mut tx, &request.space_id, &node_ids).await?;
    let mut paths =
        crate::spaces::assets::referenced_paths_for_nodes(&mut tx, &request.space_id, &node_ids)
            .await?;
    paths.extend(layout_paths(&mut tx, &request.batch_id, &structure_node_ids).await?);
    cleanup::erase(
        &mut tx,
        &request.space_id,
        &request.batch_id,
        &unit.structure,
        &node_ids,
        unit.remaining_roots,
    )
    .await?;
    tx.commit().await.map_err(db::sql_error)?;
    crate::spaces::knowledge::retrieval::invalidation_outbox::drain_and_report(app, pool).await;
    crate::spaces::assets::unreferenced_paths(pool, paths).await
}

struct DeletionUnit {
    structure: Vec<StructureNode>,
    remaining_roots: Vec<String>,
}

async fn load_unit(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    request: &PermanentlyDeleteStructureRequest,
) -> Result<DeletionUnit, String> {
    let roots_raw: String = sqlx::query_scalar(
        "SELECT root_structure_node_ids FROM spaces_deleted_structure_batches \
         WHERE id=$1 AND space_id=$2 AND restored_at IS NULL",
    )
    .bind(&request.batch_id)
    .bind(&request.space_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)?
    .ok_or("Deleted structure batch not found")?;
    let roots: Vec<String> = serde_json::from_str(&roots_raw).map_err(db::json_error)?;
    if let Some(root) = &request.root_structure_node_id {
        if !roots.contains(root) {
            return Err("Deleted root is not part of the batch".into());
        }
    }
    let rows = sqlx::query(
        "SELECT structure_node_id,data FROM spaces_deleted_structure_nodes WHERE batch_id=$1",
    )
    .bind(&request.batch_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let mut all = HashMap::new();
    for row in rows {
        let id: String = row.try_get("structure_node_id").map_err(db::sql_error)?;
        let raw: String = row.try_get("data").map_err(db::sql_error)?;
        all.insert(id, serde_json::from_str(&raw).map_err(db::json_error)?);
    }
    let selected = match request.root_structure_node_id.as_deref() {
        Some(root) => subtree(&all, root),
        None => all.keys().cloned().collect(),
    };
    let structure = selected
        .iter()
        .map(|id| all.get(id).cloned().ok_or("Deleted structure_node missing"))
        .collect::<Result<Vec<_>, _>>()?;
    let remaining_roots = roots
        .into_iter()
        .filter(|id| !selected.contains(id))
        .collect();
    Ok(DeletionUnit {
        structure,
        remaining_roots,
    })
}

fn subtree(structure: &HashMap<String, StructureNode>, root: &str) -> HashSet<String> {
    let mut selected = HashSet::new();
    let mut pending = vec![root.to_owned()];
    while let Some(id) = pending.pop() {
        if selected.insert(id.clone()) {
            pending.extend(
                structure
                    .iter()
                    .filter(|(_, structure_node)| structure_node.parent_id() == Some(id.as_str()))
                    .map(|(id, _)| id.clone()),
            );
        }
    }
    selected
}

async fn layout_paths(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    batch_id: &str,
    structure_node_ids: &HashSet<&str>,
) -> Result<BTreeSet<String>, String> {
    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT structure_node_id,data FROM spaces_deleted_structure_layouts WHERE batch_id=$1",
    )
    .bind(batch_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let mut paths = BTreeSet::new();
    for (structure_node_id, raw) in rows {
        if !structure_node_ids.contains(structure_node_id.as_str()) {
            continue;
        }
        let data: serde_json::Value = serde_json::from_str(&raw).map_err(db::json_error)?;
        if let Some(path) = data
            .get("appearance")
            .and_then(|value| value.get("background"))
            .and_then(serde_json::Value::as_str)
        {
            paths.insert(path.to_owned());
        }
    }
    Ok(paths)
}
