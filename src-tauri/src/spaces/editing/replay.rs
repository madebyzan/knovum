use crate::spaces::{db, canvas_state, nodes};
use super::{contracts::{ReplayEditRequest, ReplayDirection, EditResult, Receipt, ReceiptChanges}, changes, receipts, lifecycle_state, lifecycle_guards, lifecycle_replay, archive};
pub(super) async fn replay(pool: &sqlx::SqlitePool, request: ReplayEditRequest) -> Result<EditResult, String> {
    receipts::validate_request(&request.space_id, &request.mutation_id, request.base_revision)?;
    let hash = receipts::fingerprint(&request)?;
    let mut tx = db::begin_write(pool).await?;
    if let Some(previous) = receipts::find(&mut tx, &request.space_id, &request.mutation_id).await? {
        if previous.request_hash != hash { return Err("Mutation identity was already used for another history operation".into()); }
        let result = super::result::current(&mut tx, &request.space_id, &previous, true).await?;
        tx.commit().await.map_err(db::sql_error)?; return Ok(result);
    }
    let original = receipts::find(&mut tx, &request.space_id, &request.receipt_id).await?.ok_or("This edit receipt is no longer available")?;
    if !original.changes.reversible { return Err("This transaction is not an undoable user edit".into()); }
    let (before, revision) = canvas_state::load_in_transaction(&mut tx, &request.space_id).await?;
    if revision != request.base_revision { return Err("Space edit revision conflict".into()); }
    let inverse = matches!(request.direction, ReplayDirection::Undo);
    let after = changes::apply(&before, &original.changes.document, inverse)?;
    let expected = lifecycle_guards::direction(&original.changes.lifecycle, inverse);
    let before_lifecycle = lifecycle_state::capture(&mut tx, &request.space_id).await?;
    lifecycle_guards::validate_before(&before_lifecycle, &expected)?;
    let now = crate::clock::unix_time_millis()?;
    lifecycle_replay::apply(&mut tx, &request.space_id, &expected, now).await?;
    let after_lifecycle = lifecycle_state::capture(&mut tx, &request.space_id).await?;
    lifecycle_guards::validate_after(&before_lifecycle, &after_lifecycle, &expected)?;
    if expected.nodes.iter().any(|change| change.after.as_ref().is_none_or(|node| !node.active)) {
        archive::removed(&mut tx, &request.space_id, &request.mutation_id, &before, &after, now).await?;
    }
    archive::remove_restored(&mut tx, &request.space_id, &after).await?;
    let delta = changes::diff(&before, &after);
    let lifecycle = lifecycle_state::diff(&before_lifecycle, &after_lifecycle);
    let changed = !changes::unchanged(&delta) || !lifecycle_state::empty(&lifecycle);
    let revision = if changed { canvas_state::save_in_transaction(&mut tx, &request.space_id, revision, &after).await? } else { revision };
    nodes::touch_updated_at(&mut tx, &request.space_id, now).await?;
    let (selection_before, selection_after) = if inverse { (original.selection_after, original.selection_before) } else { (original.selection_before, original.selection_after) };
    let receipt = Receipt { mutation_id: request.mutation_id, request_hash: hash, revision, selection_before, selection_after,
        changes: ReceiptChanges { version: 1, reversible: false, document: delta, lifecycle } };
    receipts::store(&mut tx, &request.space_id, &receipt, now).await?;
    let result = super::result::current(&mut tx, &request.space_id, &receipt, false).await?;
    tx.commit().await.map_err(db::sql_error)?; Ok(result)
}
