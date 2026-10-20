use std::collections::{HashMap, HashSet};

use crate::foundation::app_error::AppError;
use sqlx::{Sqlite, Transaction};

use super::portal;
use super::{Side, TargetState};
use crate::spaces::{
    db,
    structure::{portal_integrity, repository, StructureNode},
};

pub(super) async fn apply(
    tx: &mut Transaction<'_, Sqlite>,
    targets: &[TargetState],
    requested: &str,
    side: Side,
    redo: bool,
) -> Result<i64, AppError> {
    for target in targets {
        repository::rows::vacate(tx, &target.space_id, &target.patch.structure_ids()).await?;
    }
    // A structure row that this side does not expect has left the Space for its sibling Space
    // under the same identity, so it is erased rather than kept in the Trash.
    for target in targets {
        let absent = deepest_first(
            &target.current.structure,
            &target.patch.absent_structure(side),
        );
        repository::rows::erase(tx, &target.space_id, &absent).await?;
    }
    let node_owners = owners(targets, side);
    let moved = targets
        .iter()
        .flat_map(|target| target.patch.touched_nodes())
        .collect::<Vec<_>>();
    move_nodes(tx, &node_owners).await?;
    follow_nodes(tx, &moved).await?;
    for target in targets {
        super::super::apply_structure(tx, &target.space_id, &target.patch, side, &target.current)
            .await?;
        super::super::apply_relations(tx, &target.patch, side).await?;
        super::super::apply_relation_styles(tx, &target.space_id, &target.patch, side).await?;
    }
    portal::update_targets(tx, &node_owners, &containers(targets, side), &moved).await?;
    let mut requested_next = 0;
    for target in targets {
        let (mut snapshot, _) = repository::load_in_transaction(tx, &target.space_id).await?;
        super::super::fit_restored_content(
            tx,
            &target.space_id,
            &mut snapshot,
            &target.patch.present_nodes(side),
        )
        .await?;
        let next = repository::advance_revision(tx, &target.space_id, target.revision).await?;
        super::super::replay::settle(tx, target.history_id, redo).await?;
        if target.space_id == requested {
            requested_next = next;
        }
    }
    portal_integrity::validate_structure_targets(
        tx,
        &targets
            .iter()
            .map(|target| target.space_id.as_str())
            .collect::<Vec<_>>(),
    )
    .await?;
    Ok(requested_next)
}

/// Which Space each moved node belongs to on this side: the record that expects to find it owns it.
fn owners(targets: &[TargetState], side: Side) -> HashMap<String, &str> {
    targets
        .iter()
        .flat_map(|target| {
            target
                .patch
                .present_nodes(side)
                .into_iter()
                .map(move |id| (id, target.space_id.as_str()))
        })
        .collect()
}

fn containers(targets: &[TargetState], side: Side) -> HashMap<&str, &str> {
    targets
        .iter()
        .flat_map(|target| {
            target
                .patch
                .containers(side)
                .into_iter()
                .map(move |id| (id, target.space_id.as_str()))
        })
        .collect()
}

async fn move_nodes(
    tx: &mut Transaction<'_, Sqlite>,
    owner_by_node_id: &HashMap<String, &str>,
) -> Result<(), AppError> {
    let now = crate::foundation::clock::unix_time_millis()?;
    for (node, space) in owner_by_node_id {
        sqlx::query(
            "UPDATE spaces_nodes SET space_id=$1,updated_at=$2 \
             WHERE id=$3 AND space_id<>$1 AND deleted_at IS NULL",
        )
        .bind(space)
        .bind(now)
        .bind(node)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    }
    Ok(())
}

/// Relations, Provenance and Knowledge chunks are owned by the Space of the content they describe,
/// so they follow the nodes that moved — and only those rows: the rest of the library is untouched.
async fn follow_nodes(
    tx: &mut Transaction<'_, Sqlite>,
    node_ids: &[String],
) -> Result<(), AppError> {
    if node_ids.is_empty() {
        return Ok(());
    }
    let relations: Vec<String> = sqlx::query_scalar(
        "SELECT id FROM spaces_relations WHERE source_id IN (SELECT value FROM json_each($1)) \
         UNION \
         SELECT id FROM spaces_relations WHERE target_id IN (SELECT value FROM json_each($1))",
    )
    .bind(sqlx::types::Json(node_ids))
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let now = crate::foundation::clock::unix_time_millis()?;
    sqlx::query(
        "UPDATE spaces_relations SET space_id=(SELECT space_id FROM spaces_nodes WHERE id=source_id),updated_at=$1 \
         WHERE id IN (SELECT value FROM json_each($2)) \
         AND (SELECT space_id FROM spaces_nodes WHERE id=source_id) \
           = (SELECT space_id FROM spaces_nodes WHERE id=target_id)",
    )
    .bind(now)
    .bind(sqlx::types::Json(&relations))
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    sqlx::query(
        "UPDATE spaces_provenance SET space_id=(SELECT space_id FROM spaces_nodes WHERE id=source_node_id) \
         WHERE source_node_id IN (SELECT value FROM json_each($1))",
    )
    .bind(sqlx::types::Json(node_ids))
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    sqlx::query(
        "UPDATE spaces_knowledge_chunks SET space_id=(SELECT space_id FROM spaces_nodes WHERE id=node_id),updated_at=$1 \
         WHERE node_id IN (SELECT value FROM json_each($2))",
    )
    .bind(now)
    .bind(sqlx::types::Json(node_ids))
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}

fn deepest_first(current: &[StructureNode], removed: &[String]) -> Vec<String> {
    let structure_node_by_id: HashMap<&str, &StructureNode> = current
        .iter()
        .map(|structure_node| (structure_node.id(), structure_node))
        .collect();
    let present: HashSet<&str> = structure_node_by_id.keys().copied().collect();
    let mut removed = removed
        .iter()
        .filter(|id| present.contains(id.as_str()))
        .cloned()
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
