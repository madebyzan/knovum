mod links;
mod objects;
mod state;

use serde_json::Value;
use sqlx::{Sqlite, Transaction};

use super::contract::KnowledgeChange;
use state::ApplyState;

pub(super) async fn execute(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    agent_id: &str,
    operations: &[KnowledgeChange],
    now: i64,
) -> Result<Value, String> {
    let mut state = ApplyState::default();
    objects::create(tx, space_id, agent_id, operations, now, &mut state).await?;
    objects::update(tx, space_id, agent_id, operations, now, &state).await?;
    links::create_relations(tx, space_id, agent_id, operations, now, &mut state).await?;
    links::add_provenance(tx, space_id, agent_id, operations, now, &state).await?;
    Ok(serde_json::json!({"resolvedIds": state.resolved_ids}))
}
