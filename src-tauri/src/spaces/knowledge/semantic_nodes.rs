use sqlx::{Sqlite, Transaction};

use crate::spaces::db;

pub async fn require(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    node_id: &str,
    label: &str,
) -> Result<(), String> {
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM spaces_semantic_nodes WHERE id=$1 AND space_id=$2",
    )
    .bind(node_id)
    .bind(space_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    (count == 1)
        .then_some(())
        .ok_or_else(|| format!("{label} must be an active semantic node in the same Space"))
}
