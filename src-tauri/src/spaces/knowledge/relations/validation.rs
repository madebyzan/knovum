use sqlx::{Sqlite, Transaction};

use crate::spaces::knowledge::{
    contracts::{RelationKind, RelationStatus},
    semantic_nodes,
};

const MAX_RATIONALE_CHARS: usize = 16_000;

pub(crate) fn validate_kind(kind: &str) -> Result<(), String> {
    RelationKind::parse(kind).map(|_| ())
}

pub(crate) fn validate_status(status: &str) -> Result<(), String> {
    RelationStatus::parse(status).map(|_| ())
}

pub(crate) fn validate_current_status(status: &str) -> Result<(), String> {
    match RelationStatus::parse(status)? {
        RelationStatus::Active | RelationStatus::Contested => Ok(()),
        RelationStatus::Invalidated => {
            Err("Invalidated Relations must use the dedicated lifecycle command".into())
        }
    }
}

pub(crate) fn validate_confidence(value: Option<f64>) -> Result<(), String> {
    if value.is_some_and(|confidence| !(0.0..=1.0).contains(&confidence)) {
        return Err("Relation confidence must be between 0 and 1".into());
    }
    Ok(())
}

pub(crate) fn validate_rationale(value: &str) -> Result<(), String> {
    if value.encode_utf16().count() > MAX_RATIONALE_CHARS {
        return Err(format!(
            "Relation rationale exceeds the maximum of {MAX_RATIONALE_CHARS} characters"
        ));
    }
    Ok(())
}

pub(crate) async fn validate_endpoints(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    source_id: &str,
    target_id: &str,
) -> Result<(), String> {
    if source_id == target_id {
        return Err("A relation cannot target itself".into());
    }
    semantic_nodes::require(tx, space_id, source_id, "Relation source").await?;
    semantic_nodes::require(tx, space_id, target_id, "Relation target").await
}
