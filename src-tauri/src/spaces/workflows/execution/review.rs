use serde_json::{json, Value};
use sqlx::{Row, SqlitePool};

use super::shared::waiting;
use crate::spaces::db;
use crate::spaces::runs::{StepOutcome, WaitReason};

pub(super) async fn execute(
    pool: &SqlitePool,
    space_id: &str,
    input: &Value,
    purpose: String,
) -> Result<StepOutcome, String> {
    let child_run_id = input
        .get("childRunId")
        .and_then(Value::as_str)
        .ok_or("review-input-invalid: completed Action Run is required")?;
    let row = sqlx::query(
        "SELECT d.output_node_id,n.data FROM spaces_knowledge_derivations d
         JOIN spaces_nodes n ON n.id=d.output_node_id AND n.space_id=d.space_id
         WHERE d.id=$1 AND d.space_id=$2 AND d.status='completed' AND n.deleted_at IS NULL",
    )
    .bind(child_run_id)
    .bind(space_id)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?
    .ok_or("review-input-invalid: completed resource is unavailable")?;
    let node_id: String = row.try_get("output_node_id").map_err(db::sql_error)?;
    let encoded: String = row.try_get("data").map_err(db::sql_error)?;
    let data: Value = serde_json::from_str(&encoded).map_err(db::json_error)?;
    let revision = data
        .get("knowledgeRevision")
        .or_else(|| data.get("sourceRevision"))
        .and_then(Value::as_str)
        .ok_or("review-input-invalid: a resource revision is required")?;
    waiting(
        WaitReason::ContentReview,
        json!({
            "purpose": purpose,
            "nodeId": node_id,
            "sourceRevision": revision,
        }),
    )
}
