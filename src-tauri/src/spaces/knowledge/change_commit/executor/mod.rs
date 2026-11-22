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

#[derive(Clone, Copy)]
pub(super) struct CommitContext<'a> {
    pub(super) space_id: &'a str,
    pub(super) actor: EventActor<'a>,
    pub(super) now: i64,
}

pub(super) async fn execute(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    run_id: Option<&str>,
    actor: EventActor<'_>,
    operations: &[KnowledgeChange],
    now: i64,
) -> Result<Value, AppError> {
    let citation_by_cite_id =
        state::resolve_run_citations(tx, space_id, run_id, operations).await?;
    let context = CommitContext {
        space_id,
        actor,
        now,
    };
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
    let mut state = ApplyState {
        citation_by_cite_id,
        ..ApplyState::default()
    };
    if let Some((snapshot, _, _)) = &structure_change {
        let mut placement = snapshot.clone();
        objects::create(tx, context, operations, &mut placement, &mut state).await?;
    }
    objects::update(tx, context, operations, &state).await?;
    links::create_relations(tx, context, operations, &mut state).await?;
    links::update_relations(tx, context, operations, &state).await?;
    links::add_provenance(tx, context, operations, &state).await?;
    if let Some((mut snapshot, revision, before)) = structure_change {
        structure::creation::append_content(&mut snapshot, &state.created_nodes, None)?;
        structure::repository::save_in_transaction(tx, space_id, revision, &mut snapshot).await?;
        let after = structure::history::capture(tx, space_id, &snapshot).await?;
        structure::history::record(tx, space_id, &before, &after).await?;
    }
    Ok(serde_json::json!({"resolvedIds": state.resolved_id_by_temp_id}))
}
