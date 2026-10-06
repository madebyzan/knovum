use super::super::RestoredRelation;
use crate::spaces::db;
use crate::spaces::structure::{RelationColor, RelationStyle, StructureNode};
use sqlx::{Row, Sqlite, Transaction};
use std::collections::HashSet;

pub(super) async fn load_relations(
    tx: &mut Transaction<'_, Sqlite>,
    batch_ids: &[String],
) -> Result<(Vec<RestoredRelation>, Vec<RelationStyle>), String> {
    let mut relations = Vec::new();
    let mut styles = Vec::new();
    for batch_id in batch_ids {
        let rows=sqlx::query("SELECT relation_id,status,status_before_invalidation,color FROM spaces_deleted_structure_relations WHERE batch_id=$1 ORDER BY relation_id").bind(batch_id).fetch_all(&mut **tx).await.map_err(db::sql_error)?;
        for row in rows {
            let id: String = row.try_get("relation_id").map_err(db::sql_error)?;
            relations.push(RestoredRelation {
                id: id.clone(),
                status: row.try_get("status").map_err(db::sql_error)?,
                status_before_invalidation: row
                    .try_get("status_before_invalidation")
                    .map_err(db::sql_error)?,
            });
            if let Some(color) = row
                .try_get::<Option<String>, _>("color")
                .map_err(db::sql_error)?
            {
                styles.push(RelationStyle {
                    relation_id: id,
                    color: match color.as_str() {
                        "blue" => RelationColor::Blue,
                        "green" => RelationColor::Green,
                        "amber" => RelationColor::Amber,
                        "red" => RelationColor::Red,
                        "purple" => RelationColor::Purple,
                        _ => return Err("Deleted relation style color is invalid".into()),
                    },
                });
            }
        }
    }
    Ok((relations, styles))
}

pub(crate) async fn mark_restored(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    batch_ids: &[String],
    now: i64,
) -> Result<(), String> {
    for batch in batch_ids {
        let restoration_write=sqlx::query("UPDATE spaces_deleted_structure_batches SET restored_at=$1 WHERE id=$2 AND space_id=$3 AND restored_at IS NULL").bind(now).bind(batch).bind(space_id).execute(&mut **tx).await.map_err(db::sql_error)?;
        if restoration_write.rows_affected() != 1 {
            return Err("Deleted structure batch changed during restore".into());
        }
    }
    Ok(())
}

pub(super) async fn reject_restore_conflicts(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    structure: &[StructureNode],
) -> Result<(), String> {
    let active: Vec<String> = sqlx::query_scalar(
        "SELECT id FROM spaces_structure_nodes WHERE space_id=$1 AND deleted_at IS NULL",
    )
    .bind(space_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let active_ids: HashSet<&str> = active.iter().map(String::as_str).collect();
    let restoring: HashSet<&str> = structure.iter().map(StructureNode::id).collect();
    for structure_node in structure {
        if active_ids.contains(structure_node.id()) {
            return Err(format!(
                "Restore structure_node already active: {}",
                structure_node.id()
            ));
        }
        if let Some(parent) = structure_node.parent_id() {
            if !active_ids.contains(parent) && !restoring.contains(parent) {
                return Err(format!("Restore parent is missing: {parent}"));
            }
        }
    }
    Ok(())
}

pub(crate) async fn remove_tombstones(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    structure: &[StructureNode],
) -> Result<(), String> {
    // Parent foreign keys are deferred; snapshot publication restores the identities before commit.
    let ids = serde_json::to_string(&structure.iter().map(StructureNode::id).collect::<Vec<_>>())
        .map_err(db::json_error)?;
    sqlx::query("DELETE FROM spaces_structure_layouts WHERE space_id=$1 AND structure_node_id IN (SELECT value FROM json_each($2))")
        .bind(space_id).bind(&ids).execute(&mut **tx).await.map_err(db::sql_error)?;
    sqlx::query("DELETE FROM spaces_structure_nodes WHERE space_id=$1 AND deleted_at IS NOT NULL AND id IN (SELECT value FROM json_each($2))")
        .bind(space_id).bind(&ids).execute(&mut **tx).await.map_err(db::sql_error)?;
    Ok(())
}
