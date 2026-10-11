use serde_json::Value;
use sqlx::{Sqlite, Transaction};

use super::change_commit::{self, KnowledgeChange};
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
    change_commit::validate_structure(&operations)?;
    let exists = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM spaces WHERE id=$1")
        .bind(space_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    if exists != 1 {
        return Err("Knowledge Change Set target Space does not exist".into());
    }
    change_commit::validate_references(tx, space_id, &operations).await?;
    let actor = super::events::EventActor::new(
        principal.kind.event_actor_kind(),
        Some(principal.id.as_str()),
    );
    change_commit::apply(tx, space_id, actor, operations, now).await
}
