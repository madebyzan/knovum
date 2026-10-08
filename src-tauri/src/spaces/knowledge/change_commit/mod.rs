use crate::app_error::AppError;
mod contracts;
mod executor;
mod mutations;
mod validation;

use serde_json::Value;
use sqlx::{Sqlite, Transaction};

use super::events::EventActor;

pub(crate) use contracts::KnowledgeChange;
pub(crate) use validation::{validate_references, validate_structure};

pub async fn apply(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    actor: EventActor<'_>,
    operations: Vec<KnowledgeChange>,
    now: i64,
) -> Result<Value, AppError> {
    validate_structure(&operations)?;
    super::events::validation::validate_actor_kind(actor.kind())?;
    executor::execute(tx, space_id, actor, &operations, now).await
}
