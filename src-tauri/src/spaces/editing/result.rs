use std::collections::HashSet;
use crate::spaces::{canvas_state, queries};
use super::{contracts::{Receipt, EditResult}, lifecycle_state};
pub(super) async fn current(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, space_id: &str, receipt: &Receipt, replayed: bool) -> Result<EditResult, String> {
    let (document, revision) = canvas_state::load_in_transaction(tx, space_id).await?;
    canvas_state::validation::validate_content_references(tx, space_id, &document).await?;
    let ids: HashSet<_> = document.structure.items.iter().map(|item| item.id.as_str()).collect();
    let selection = receipt.selection_after.iter().filter(|id| ids.contains(id.as_str())).cloned().collect();
    let content = if replayed || !lifecycle_state::empty(&receipt.changes.lifecycle) { Some(queries::read_editor_content(tx, space_id).await?) } else { None };
    Ok(EditResult { document, revision, applied_at_revision: receipt.revision, mutation_id: receipt.mutation_id.clone(), selection,
        history_receipt_id: receipt.changes.reversible.then(|| receipt.mutation_id.clone()), content })
}
