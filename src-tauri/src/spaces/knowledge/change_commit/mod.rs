use crate::app_error::AppError;
mod contract;
mod executor;
mod mutations;

use serde_json::Value;
use sqlx::{Sqlite, Transaction};

use super::events::EventActor;

pub(crate) use contract::KnowledgeChange;
pub(crate) use contract::{validate_references, validate_structure};

pub async fn apply(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    actor: EventActor<'_>,
    operations: Vec<KnowledgeChange>,
    now: i64,
) -> Result<Value, AppError> {
    validate_structure(&operations)?;
    super::contracts::validate_actor_kind(actor.kind())?;
    executor::execute(tx, space_id, actor, &operations, now).await
}
