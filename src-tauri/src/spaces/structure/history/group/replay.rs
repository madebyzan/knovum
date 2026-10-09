use crate::app_error::AppError;
use std::collections::{HashMap, HashSet};

use sqlx::{Sqlite, Transaction};

use super::TargetState;
use crate::spaces::{db, structure::StructureNode};

pub(super) fn preserve_history_neutral_content_sizes(targets: &mut [TargetState]) {
    let current = super::super::neutral::content_size_states(
        targets.iter().map(|target| &target.current_snapshot),
    );
    let before = super::super::neutral::content_size_states(
        targets.iter().map(|target| &target.before_snapshot),
    );
    let after = super::super::neutral::content_size_states(
        targets.iter().map(|target| &target.after_snapshot),
    );
    for target in targets {
        super::super::neutral::preserve_content_sizes(
            &mut target.snapshot,
            &current,
            &before,
            &after,
        );
    }
}

pub(super) async fn remove_current_structures(
    tx: &mut Transaction<'_, Sqlite>,
    targets: &[TargetState],
) -> Result<(), AppError> {
    for target in targets {
        let (current, _) =
            super::super::super::repository::load_in_transaction(tx, &target.space_id).await?;
        let absent = removed_structure_deepest(&current.structure, &target.snapshot.structure);
        super::super::super::repository::remove_transferred_structure_nodes_in_transaction(
            tx,
            &target.space_id,
            &absent,
        )
        .await?;
        sqlx::query("DELETE FROM spaces_relation_styles WHERE space_id=$1")
            .bind(&target.space_id)
            .execute(&mut **tx)
            .await
            .map_err(db::sql_error)?;
    }
    Ok(())
}

pub(super) async fn move_nodes(
    tx: &mut Transaction<'_, Sqlite>,
    owners: &HashMap<&str, &str>,
) -> Result<(), AppError> {
    for (node, space) in owners {
        sqlx::query("UPDATE spaces_nodes SET space_id=$1 WHERE id=$2 AND deleted_at IS NULL")
            .bind(space)
            .bind(node)
            .execute(&mut **tx)
            .await
            .map_err(db::sql_error)?;
    }
    Ok(())
}

pub(super) async fn reconcile_knowledge_ownership(
    tx: &mut Transaction<'_, Sqlite>,
    targets: &[TargetState],
) -> Result<(), AppError> {
    let ids: HashSet<&str> = targets
        .iter()
        .flat_map(|target| target.relations.iter().map(|relation| relation.id.as_str()))
        .collect();
    let now = crate::clock::unix_time_millis()?;
    for id in ids {
        sqlx::query("UPDATE spaces_relations SET space_id=(SELECT space_id FROM spaces_nodes WHERE id=source_id),updated_at=$1 WHERE id=$2 AND (SELECT space_id FROM spaces_nodes WHERE id=source_id)=(SELECT space_id FROM spaces_nodes WHERE id=target_id)").bind(now).bind(id).execute(&mut **tx).await.map_err(db::sql_error)?;
    }
    sqlx::query("UPDATE spaces_provenance SET space_id=(SELECT space_id FROM spaces_nodes WHERE id=source_node_id) WHERE source_node_id IN (SELECT id FROM spaces_nodes)").execute(&mut **tx).await.map_err(db::sql_error)?;
    sqlx::query("UPDATE spaces_knowledge_chunks SET space_id=(SELECT space_id FROM spaces_nodes WHERE id=node_id) WHERE node_id IN (SELECT id FROM spaces_nodes)").execute(&mut **tx).await.map_err(db::sql_error)?;
    Ok(())
}

pub(super) async fn save_targets(
    tx: &mut Transaction<'_, Sqlite>,
    targets: &[TargetState],
    requested: &str,
    redo: bool,
) -> Result<i64, AppError> {
    let mut requested_next = 0;
    for target in targets {
        super::super::apply_relations(tx, &target.relations).await?;
        let revision = super::current_revision(tx, &target.space_id).await?;
        let next = super::super::super::repository::save_in_transaction(
            tx,
            &target.space_id,
            revision,
            &target.snapshot,
        )
        .await?;
        sqlx::query("UPDATE spaces_edit_history SET undone=$1,expected_revision=$2 WHERE id=$3")
            .bind(if redo { 0 } else { 1 })
            .bind(next)
            .bind(target.history_id)
            .execute(&mut **tx)
            .await
            .map_err(db::sql_error)?;
        if target.space_id == requested {
            requested_next = next;
        }
    }
    super::super::super::portal_integrity::validate_structure_targets(
        tx,
        &targets
            .iter()
            .map(|target| target.space_id.as_str())
            .collect::<Vec<_>>(),
    )
    .await?;
    Ok(requested_next)
}

fn removed_structure_deepest(current: &[StructureNode], desired: &[StructureNode]) -> Vec<String> {
    let desired: HashSet<&str> = desired.iter().map(StructureNode::id).collect();
    let structure_node_by_id: HashMap<&str, &StructureNode> = current
        .iter()
        .map(|structure_node| (structure_node.id(), structure_node))
        .collect();
    let mut removed = current
        .iter()
        .filter(|structure_node| !desired.contains(structure_node.id()))
        .map(|structure_node| structure_node.id().to_owned())
        .collect::<Vec<_>>();
    removed.sort_by_key(|id| std::cmp::Reverse(depth(id, &structure_node_by_id)));
    removed
}

fn depth(structure_node_id: &str, structure: &HashMap<&str, &StructureNode>) -> usize {
    let mut depth = 0;
    let mut parent = structure
        .get(structure_node_id)
        .and_then(|structure_node| structure_node.parent_id());
    while let Some(value) = parent {
        depth += 1;
        parent = structure
            .get(value)
            .and_then(|structure_node| structure_node.parent_id());
    }
    depth
}
