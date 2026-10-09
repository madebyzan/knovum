use std::collections::HashSet;

use sqlx::{Sqlite, Transaction};

use super::{
    edit_contract::ExecuteSpaceEditRequest,
    planner::delete::Deletion,
    repository::{self, SpaceStructureSnapshot},
};
use crate::spaces::db;

pub(super) async fn apply(
    tx: &mut Transaction<'_, Sqlite>,
    request: &ExecuteSpaceEditRequest,
    snapshot: &mut SpaceStructureSnapshot,
    deletion: &Deletion,
) -> Result<(), String> {
    let now = crate::clock::unix_time_millis()?;
    repository::store_deletion_in_transaction(
        tx,
        repository::DeletionBatch {
            space_id: &request.space_id,
            batch_id: &request.mutation_id,
            roots: &deletion.root_ids,
            structure: &deletion.structure,
            layouts: &deletion.layouts,
            node_ids: &deletion.node_ids,
            deleted_at: now,
        },
    )
    .await?;
    let trashed =
        crate::spaces::nodes::trash_in_transaction(tx, &request.space_id, &deletion.node_ids, now)
            .await?;
    require_exact_nodes(&deletion.node_ids, &trashed)?;
    let invalidated: Vec<String> = sqlx::query_scalar(
        "SELECT id FROM spaces_relations WHERE space_id=$1 AND status='invalidated' AND (source_id IN (SELECT value FROM json_each($2)) OR target_id IN (SELECT value FROM json_each($2)))",
    )
    .bind(&request.space_id)
    .bind(sqlx::types::Json(&deletion.node_ids))
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let invalidated: HashSet<String> = invalidated.into_iter().collect();
    snapshot
        .relation_styles
        .retain(|style| !invalidated.contains(&style.relation_id));
    Ok(())
}

fn require_exact_nodes(expected: &[String], actual: &[String]) -> Result<(), String> {
    let expected: HashSet<&str> = expected.iter().map(String::as_str).collect();
    let actual: HashSet<&str> = actual.iter().map(String::as_str).collect();
    if expected == actual {
        Ok(())
    } else {
        Err("Deletion node set changed during transaction".into())
    }
}
