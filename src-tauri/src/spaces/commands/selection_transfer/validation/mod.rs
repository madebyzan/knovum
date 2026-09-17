mod boundary;
mod node_contracts;

use std::collections::HashSet;

use super::contracts::TransferSelectionRequest;
use crate::spaces::db;

pub(super) async fn ensure_target_space(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
) -> Result<(), String> {
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM spaces WHERE id=$1 AND kind IN ('space','inbox') AND archived_at IS NULL",
    )
    .bind(space_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    (count == 1)
        .then_some(())
        .ok_or_else(|| "Target Space does not exist or is archived".into())
}

pub(super) async fn selected_nodes(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    request: &TransferSelectionRequest,
) -> Result<Vec<String>, String> {
    let unique = super::super::dedup_and_validate_node_ids(
        &request.ids,
        "Space move item ID cannot be empty",
    )?;
    let requested_json = serde_json::to_string(&unique).map_err(db::json_error)?;
    let rows = sqlx::query_scalar::<_, String>(
        "SELECT id FROM spaces_nodes WHERE space_id=$1 AND deleted_at IS NULL AND kind <> 'section' \
         AND id IN (SELECT value FROM json_each($2))",
    )
    .bind(&request.source_space_id)
    .bind(requested_json)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let found: HashSet<String> = rows.into_iter().collect();
    if found.len() != unique.len() {
        let missing = unique
            .iter()
            .filter(|id| !found.contains(*id))
            .cloned()
            .collect::<Vec<_>>();
        return Err(format!(
            "Cannot move items because these IDs are not active movable nodes in the source Space: {}",
            missing.join(", ")
        ));
    }
    Ok(unique)
}

pub(super) async fn validate_dependencies(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    selected: &[String],
) -> Result<(), String> {
    let selected_set: HashSet<&str> = selected.iter().map(String::as_str).collect();
    node_contracts::validate(tx, space_id, &selected_set).await?;
    boundary::validate_relations(tx, space_id, &selected_set).await?;
    boundary::validate_provenance(tx, space_id, &selected_set).await
}
