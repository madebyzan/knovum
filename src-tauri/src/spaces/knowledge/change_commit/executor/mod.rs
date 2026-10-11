mod links;
mod objects;
mod state;

use serde_json::Value;
use sqlx::{Sqlite, Transaction};

use super::contracts::KnowledgeChange;
use crate::{
    foundation::app_error::AppError,
    spaces::{knowledge::events::EventActor, structure},
};
use state::ApplyState;

pub(super) async fn execute(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    actor: EventActor<'_>,
    operations: &[KnowledgeChange],
    now: i64,
) -> Result<Value, AppError> {
    let structure_change = if operations
        .iter()
        .any(|operation| matches!(operation, KnowledgeChange::CreateObject { .. }))
    {
        let (snapshot, revision) = structure::repository::load_in_transaction(tx, space_id).await?;
        let before = structure::history::capture(tx, space_id, &snapshot).await?;
        Some((snapshot, revision, before))
    } else {
        None
    };
    let mut state = ApplyState::default();
    if let Some((snapshot, _, _)) = &structure_change {
        let mut placement = snapshot.clone();
        objects::create(
            tx,
            space_id,
            actor,
            operations,
            now,
            &mut placement,
            &mut state,
        )
        .await?;
    }
    objects::update(tx, space_id, actor, operations, now, &state).await?;
    links::create_relations(tx, space_id, actor, operations, now, &mut state).await?;
    links::add_provenance(tx, space_id, actor, operations, now, &state).await?;
    if let Some((mut snapshot, revision, before)) = structure_change {
        structure::creation::append_content(&mut snapshot, &state.created_nodes, None)?;
        let edit_revision =
            structure::repository::save_in_transaction(tx, space_id, revision, &snapshot).await?;
        let after = structure::history::capture(tx, space_id, &snapshot).await?;
        structure::history::record(tx, space_id, &before, &after, None, None, edit_revision)
            .await?;
    }
    Ok(serde_json::json!({"resolvedIds": state.resolved_id_by_temp_id}))
}
