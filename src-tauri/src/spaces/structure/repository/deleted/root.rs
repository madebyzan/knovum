use crate::spaces::db;
use crate::spaces::structure::StructureNode;
use sqlx::{Row, Sqlite, Transaction};
use std::collections::{HashMap, HashSet};

use super::{restore::prepare_restore_batch, RestoredStructure};

pub(crate) async fn prepare_restore_root(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    batch_id: &str,
    root_id: &str,
    new_batch_id: &str,
) -> Result<RestoredStructure, String> {
    let roots_raw:String=sqlx::query_scalar("SELECT root_structure_node_ids FROM spaces_deleted_structure_batches WHERE id=$1 AND space_id=$2 AND restored_at IS NULL").bind(batch_id).bind(space_id).fetch_optional(&mut **tx).await.map_err(db::sql_error)?.ok_or("Deleted structure batch not found")?;
    let mut roots: Vec<String> = serde_json::from_str(&roots_raw).map_err(db::json_error)?;
    if !roots.iter().any(|id| id == root_id) {
        return Err("Deleted root is not part of the batch".into());
    }
    if roots.len() == 1 {
        return prepare_restore_batch(tx, space_id, batch_id).await;
    }
    let rows = sqlx::query(
        "SELECT structure_node_id,data FROM spaces_deleted_structure_nodes WHERE batch_id=$1",
    )
    .bind(batch_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let mut decoded = HashMap::new();
    let mut children: HashMap<String, Vec<String>> = HashMap::new();
    for row in rows {
        let id: String = row.try_get("structure_node_id").map_err(db::sql_error)?;
        let raw: String = row.try_get("data").map_err(db::sql_error)?;
        let node: StructureNode = serde_json::from_str(&raw).map_err(db::json_error)?;
        if let Some(parent) = node.parent_id() {
            children
                .entry(parent.to_owned())
                .or_default()
                .push(id.clone());
        }
        decoded.insert(id, node);
    }
    if !decoded.contains_key(root_id) {
        return Err("Deleted root has no structure record".into());
    }
    let mut selected = HashSet::new();
    let mut pending = vec![root_id.to_owned()];
    while let Some(id) = pending.pop() {
        if !selected.insert(id.clone()) {
            continue;
        }
        if let Some(descendants) = children.remove(&id) {
            pending.extend(descendants);
        }
    }
    let deleted_at: i64 =
        sqlx::query_scalar("SELECT deleted_at FROM spaces_deleted_structure_batches WHERE id=$1")
            .bind(batch_id)
            .fetch_one(&mut **tx)
            .await
            .map_err(db::sql_error)?;
    sqlx::query("INSERT INTO spaces_deleted_structure_batches(id,space_id,root_structure_node_ids,deleted_at) VALUES($1,$2,$3,$4)").bind(new_batch_id).bind(space_id).bind(serde_json::to_string(&vec![root_id]).map_err(db::json_error)?).bind(deleted_at).execute(&mut **tx).await.map_err(db::sql_error)?;
    let selected_json = serde_json::to_string(&selected).map_err(db::json_error)?;
    sqlx::query("INSERT INTO spaces_deleted_structure_nodes(batch_id,structure_node_id,data) SELECT $1,structure_node_id,data FROM spaces_deleted_structure_nodes WHERE batch_id=$2 AND structure_node_id IN (SELECT value FROM json_each($3))")
        .bind(new_batch_id).bind(batch_id).bind(&selected_json).execute(&mut **tx).await.map_err(db::sql_error)?;
    sqlx::query("INSERT INTO spaces_deleted_structure_layouts(batch_id,structure_node_id,data) SELECT $1,structure_node_id,data FROM spaces_deleted_structure_layouts WHERE batch_id=$2 AND structure_node_id IN (SELECT value FROM json_each($3))")
        .bind(new_batch_id).bind(batch_id).bind(&selected_json).execute(&mut **tx).await.map_err(db::sql_error)?;
    let node_ids: HashSet<String> = selected
        .iter()
        .filter_map(|id| match &decoded[id] {
            StructureNode::Content { node_id, .. } => Some(node_id.clone()),
            _ => None,
        })
        .collect();
    let relation_rows=sqlx::query("SELECT d.relation_id,r.source_id,r.target_id FROM spaces_deleted_structure_relations d JOIN spaces_relations r ON r.id=d.relation_id WHERE d.batch_id=$1").bind(batch_id).fetch_all(&mut **tx).await.map_err(db::sql_error)?;
    for row in relation_rows {
        let relation_id: String = row.try_get("relation_id").map_err(db::sql_error)?;
        let source: String = row.try_get("source_id").map_err(db::sql_error)?;
        let target: String = row.try_get("target_id").map_err(db::sql_error)?;
        let touches = node_ids.contains(&source) || node_ids.contains(&target);
        if !touches {
            continue;
        }
        let other = if node_ids.contains(&source) {
            &target
        } else {
            &source
        };
        let other_deleted: bool =
            sqlx::query_scalar("SELECT deleted_at IS NOT NULL FROM spaces_nodes WHERE id=$1")
                .bind(other)
                .fetch_one(&mut **tx)
                .await
                .map_err(db::sql_error)?;
        if other_deleted && !node_ids.contains(other) {
            return Err("Single-root restore would leave a related endpoint deleted; restore the whole batch".into());
        }
        sqlx::query("INSERT INTO spaces_deleted_structure_relations SELECT $1,relation_id,status,status_before_invalidation,color FROM spaces_deleted_structure_relations WHERE batch_id=$2 AND relation_id=$3").bind(new_batch_id).bind(batch_id).bind(&relation_id).execute(&mut **tx).await.map_err(db::sql_error)?;
        sqlx::query(
            "DELETE FROM spaces_deleted_structure_relations WHERE batch_id=$1 AND relation_id=$2",
        )
        .bind(batch_id)
        .bind(relation_id)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    }
    sqlx::query("DELETE FROM spaces_deleted_structure_layouts WHERE batch_id=$1 AND structure_node_id IN (SELECT value FROM json_each($2))")
        .bind(batch_id).bind(&selected_json).execute(&mut **tx).await.map_err(db::sql_error)?;
    sqlx::query("DELETE FROM spaces_deleted_structure_nodes WHERE batch_id=$1 AND structure_node_id IN (SELECT value FROM json_each($2))")
        .bind(batch_id).bind(&selected_json).execute(&mut **tx).await.map_err(db::sql_error)?;
    roots.retain(|id| id != root_id);
    sqlx::query(
        "UPDATE spaces_deleted_structure_batches SET root_structure_node_ids=$1 WHERE id=$2",
    )
    .bind(serde_json::to_string(&roots).map_err(db::json_error)?)
    .bind(batch_id)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    prepare_restore_batch(tx, space_id, new_batch_id).await
}
