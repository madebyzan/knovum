mod node_selection;

use std::collections::HashSet;

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

pub(super) async fn validate_nodes(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    selected: &HashSet<&str>,
) -> Result<(), String> {
    node_selection::validate(tx, space_id, selected).await
}
