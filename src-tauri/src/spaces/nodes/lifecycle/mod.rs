mod effects;
mod validation;

use effects::{mark_nodes_deleted, restore_nodes};

pub(in crate::spaces) async fn trash_in_transaction(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    requested_ids: &[String],
    now: i64,
) -> Result<Vec<String>, String> {
    let ids = validation::active_unique_ids(tx, space_id, requested_ids, true).await?;
    if !ids.is_empty() {
        mark_nodes_deleted(tx, space_id, &ids, now).await?;
        crate::spaces::knowledge::retrieval::invalidation_outbox::enqueue_nodes(
            tx, space_id, &ids, now,
        )
        .await?;
    }
    Ok(ids)
}

pub(in crate::spaces) async fn restore_in_transaction(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    requested_ids: &[String],
    now: i64,
) -> Result<Vec<String>, String> {
    let ids = validation::active_unique_ids(tx, space_id, requested_ids, false).await?;
    if !ids.is_empty() {
        restore_nodes(tx, space_id, &ids, now).await?;
    }
    Ok(ids)
}
