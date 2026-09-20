use crate::spaces::{db, canvas_state, organization::{self, EditorCommand}, nodes};
use super::{contracts::{ExecuteEditRequest, EditResult, Receipt, ReceiptChanges}, changes, receipts, lifecycle_state, archive};
pub(super) async fn execute(pool: &sqlx::SqlitePool, request: ExecuteEditRequest) -> Result<EditResult, String> {
    receipts::validate_request(&request.space_id, &request.mutation_id, request.base_revision)?;
    let hash = receipts::fingerprint(&request)?;
    let mut tx = db::begin_write(pool).await?;
    if let Some(receipt) = receipts::find(&mut tx, &request.space_id, &request.mutation_id).await? {
        if receipt.request_hash != hash { return Err("A mutation identity cannot be reused for a different edit".into()); }
        let result = super::result::current(&mut tx, &request.space_id, &receipt, true).await?;
        tx.commit().await.map_err(db::sql_error)?; return Ok(result);
    }
    let (before, revision) = canvas_state::load_in_transaction(&mut tx, &request.space_id).await?;
    canvas_state::validation::validate_content_references(&mut tx, &request.space_id, &before).await?;
    if revision != request.base_revision { return Err("Space edit revision conflict".into()); }
    let before_state = if request.command.deletes_subtrees() { lifecycle_state::capture(&mut tx, &request.space_id).await? } else { lifecycle_state::LifecycleState::default() };
    let mut plan = match &request.command {
        EditorCommand::MeasureText { measurements } => organization::Plan { after: super::measurements::apply(&mut tx, &request.space_id, &before, measurements).await?, selection: vec![], removed_content_ids: vec![] },
        _ => organization::plan_editor(&before, &request.command)?,
    };
    let now = crate::clock::unix_time_millis()?;
    if request.command.deletes_subtrees() {
        super::lifecycle_edit::delete(&mut tx, &request.space_id, &plan.removed_content_ids, request.command.requested_relation_deletions(), now).await?;
        archive::removed(&mut tx, &request.space_id, &request.mutation_id, &before, &plan.after, now).await?;
        let live: Vec<String> = sqlx::query_scalar("SELECT id FROM spaces_relations WHERE space_id=$1 AND status<>'invalidated'").bind(&request.space_id).fetch_all(&mut *tx).await.map_err(db::sql_error)?;
        if let Some(styles) = plan.after.relation_styles.as_mut() { styles.retain(|style| live.contains(&style.relation_id)); }
    }
    let lifecycle = if request.command.deletes_subtrees() { lifecycle_state::diff(&before_state, &lifecycle_state::capture(&mut tx, &request.space_id).await?) } else { Default::default() };
    let delta = changes::diff(&before, &plan.after);
    let changed = !changes::unchanged(&delta) || !lifecycle_state::empty(&lifecycle);
    let revision = if changed { canvas_state::save_in_transaction(&mut tx, &request.space_id, revision, &plan.after).await? } else { revision };
    if changed { nodes::touch_updated_at(&mut tx, &request.space_id, now).await?; }
    let selection_before = match &request.command { EditorCommand::Structure { selection, .. } => selection.clone(), EditorCommand::DeleteSelection { ids, .. } => ids.clone(), EditorCommand::TransformFrames { updates } | EditorCommand::ArrangeFrames { updates } => updates.iter().map(|update| update.node_id.clone()).collect(), _ => vec![] };
    let receipt = Receipt { mutation_id: request.mutation_id, request_hash: hash, revision, selection_before, selection_after: plan.selection,
        changes: ReceiptChanges { version: 1, reversible: changed && !matches!(request.command, EditorCommand::MeasureText { .. }), document: delta, lifecycle },
    };
    receipts::store(&mut tx, &request.space_id, &receipt, now).await?;
    let result = super::result::current(&mut tx, &request.space_id, &receipt, false).await?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(result)
}
