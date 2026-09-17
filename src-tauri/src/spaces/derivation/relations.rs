use uuid::Uuid;

use super::DerivationRelation;
use crate::spaces::commands::relations::SpaceRelationInput;

pub fn derives_from(
    space_id: &str,
    output_id: &str,
    source_id: &str,
    rationale: &'static str,
    now: i64,
) -> DerivationRelation {
    DerivationRelation {
        id: Uuid::new_v4().to_string(),
        space_id: space_id.into(),
        source_id: output_id.into(),
        target_id: source_id.into(),
        kind: "derives-from",
        rationale,
        status: "active",
        actor_kind: "system",
        created_at: now,
        updated_at: now,
    }
}

pub async fn insert(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    value: &DerivationRelation,
) -> Result<(), String> {
    let relation = SpaceRelationInput {
        id: value.id.clone(),
        space_id: value.space_id.clone(),
        source_id: value.source_id.clone(),
        target_id: value.target_id.clone(),
        kind: value.kind.to_owned(),
        rationale: value.rationale.to_owned(),
        status: value.status.to_owned(),
        confidence: None,
    };
    crate::spaces::commands::relations::create_relation_in_transaction(
        tx,
        &relation,
        value.actor_kind,
        None,
        value.created_at,
    )
    .await
}
