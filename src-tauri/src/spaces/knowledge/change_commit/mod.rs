mod contract;
mod executor;
mod mutations;

use serde_json::Value;
use sqlx::{Sqlite, Transaction};

pub(crate) use contract::KnowledgeChange;
pub(crate) use contract::{validate_references, validate_structure};

pub async fn apply(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    agent_id: &str,
    operations: Vec<KnowledgeChange>,
    now: i64,
) -> Result<Value, String> {
    validate_structure(&operations)?;
    executor::execute(tx, space_id, agent_id, &operations, now).await
}
