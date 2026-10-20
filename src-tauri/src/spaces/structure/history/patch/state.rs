use std::collections::{BTreeMap, BTreeSet};

use sqlx::{Row, Sqlite, Transaction};

use super::super::RelationLifecycle;
use crate::spaces::{
    db,
    structure::{
        repository::{rows::StructureRow, SpaceStructureSnapshot},
        RelationColor, StructureNode,
    },
};

/// The Space state an edit is measured against.
///
/// Structure, layout and style come from the snapshot the edit already holds, so capturing them
/// costs nothing. Relation lifecycle lives only in the relation rows — invalidation rewrites it as
/// a side effect of removing content — so it is read here while it still describes the state
/// before the edit.
pub(crate) struct HistoryState {
    pub(super) structure: BTreeMap<String, StructureRow>,
    pub(super) relation_styles: BTreeMap<String, RelationColor>,
    pub(super) relations: BTreeMap<String, RelationLifecycle>,
}

impl HistoryState {
    /// Live nodes and active content references are the same set — every save proves it — so the
    /// snapshot already says which nodes this Space holds.
    pub(super) fn live_nodes(&self) -> BTreeSet<String> {
        self.structure
            .values()
            .filter_map(|row| match &row.node {
                StructureNode::Content { node_id, .. } => Some(node_id.clone()),
                _ => None,
            })
            .collect()
    }
}

pub(crate) async fn capture_state(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    snapshot: &SpaceStructureSnapshot,
) -> Result<HistoryState, String> {
    let mut layouts = snapshot
        .layouts
        .iter()
        .map(|layout| (layout.structure_node_id(), layout))
        .collect::<BTreeMap<_, _>>();
    let mut structure = BTreeMap::new();
    for node in &snapshot.structure {
        let layout = layouts
            .remove(node.id())
            .ok_or_else(|| format!("Structure node {} has no layout", node.id()))?;
        structure.insert(
            node.id().to_owned(),
            StructureRow {
                node: node.clone(),
                layout: layout.clone(),
            },
        );
    }
    Ok(HistoryState {
        structure,
        relation_styles: snapshot
            .relation_styles
            .iter()
            .map(|style| (style.relation_id.clone(), style.color.clone()))
            .collect(),
        relations: relations(tx, space_id).await?,
    })
}

async fn relations(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
) -> Result<BTreeMap<String, RelationLifecycle>, String> {
    let rows = sqlx::query(
        "SELECT id,status,status_before_invalidation FROM spaces_relations WHERE space_id=$1",
    )
    .bind(space_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    rows.into_iter()
        .map(|row| {
            Ok((
                row.try_get("id").map_err(db::sql_error)?,
                RelationLifecycle {
                    status: row.try_get("status").map_err(db::sql_error)?,
                    status_before_invalidation: row
                        .try_get("status_before_invalidation")
                        .map_err(db::sql_error)?,
                },
            ))
        })
        .collect()
}
