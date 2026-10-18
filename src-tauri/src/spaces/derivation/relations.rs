use uuid::Uuid;

use crate::spaces::commands::relations::{create_relation_in_transaction, SpaceRelationInput};
use crate::spaces::knowledge::contracts::{KnowledgeActorKind, RelationKind, RelationStatus};

/// Record that `output_id` derives from `source_id`, in the caller's transaction, so a derived
/// node and its lineage reach the canvas together. `lineage` says what the derivation did.
pub async fn derives_from(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    output_id: &str,
    source_id: &str,
    lineage: &'static str,
    now: i64,
) -> Result<(), String> {
    let relation = SpaceRelationInput {
        id: Uuid::new_v4().to_string(),
        space_id: space_id.into(),
        source_id: output_id.into(),
        target_id: source_id.into(),
        kind: RelationKind::DerivesFrom,
        rationale: lineage.to_owned(),
        status: RelationStatus::Active,
        confidence: None,
    };
    create_relation_in_transaction(
        tx,
        &relation,
        KnowledgeActorKind::System.as_str(),
        None,
        now,
    )
    .await
}
