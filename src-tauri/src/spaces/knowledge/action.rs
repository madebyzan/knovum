use serde_json::Value;
use sqlx::{Sqlite, Transaction};

use super::{
    change_commit::{self, KnowledgeChange},
    events::EventActor,
};
use crate::{
    foundation::app_error::AppError,
    spaces::{authorization::Principal, db},
};

pub(crate) async fn commit_changes(
    tx: &mut Transaction<'_, Sqlite>,
    principal: &Principal,
    space_id: &str,
    parameters: &Value,
    now: i64,
) -> Result<Value, AppError> {
    let operations = parameters
        .get("operations")
        .cloned()
        .ok_or("commit-knowledge-changes requires operations")?;
    let operations: Vec<KnowledgeChange> =
        serde_json::from_value(operations).map_err(db::json_error)?;
    let actor = EventActor::new(
        principal.kind.event_actor_kind(),
        Some(principal.id.as_str()),
    );
    commit(tx, actor, space_id, operations, now).await
}

/// Commits one Change Set atomically. Every Knowledge mutation goes through here: an Agent's
/// Action and an accepted Knowledge Proposal alike.
pub(crate) async fn commit(
    tx: &mut Transaction<'_, Sqlite>,
    actor: EventActor<'_>,
    space_id: &str,
    operations: Vec<KnowledgeChange>,
    now: i64,
) -> Result<Value, AppError> {
    validate(tx, space_id, &operations).await?;
    change_commit::apply(tx, space_id, actor, operations, now).await
}

/// Everything a commit checks before it writes. A proposal is checked the same way when it is
/// made, so review never offers a Change Set that cannot be accepted.
pub(crate) async fn validate(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    operations: &[KnowledgeChange],
) -> Result<(), AppError> {
    change_commit::validate_structure(operations)?;
    let exists = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM spaces WHERE id=$1")
        .bind(space_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    if exists != 1 {
        return Err("Knowledge Change Set target Space does not exist".into());
    }
    change_commit::validate_references(tx, space_id, operations).await?;
    Ok(())
}
