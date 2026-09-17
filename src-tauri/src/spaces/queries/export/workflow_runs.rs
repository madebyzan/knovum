use serde_json::Value;
use sqlx::{Row, Sqlite, Transaction};

use crate::spaces::db;

pub async fn rows(tx: &mut Transaction<'_, Sqlite>, space_id: &str) -> Result<Vec<Value>, String> {
    super::reject_running(
        tx,
        "spaces_creation_workflow_runs",
        space_id,
        "Wait for active Creation Workflows to finish before exporting",
    )
    .await?;
    let rows = sqlx::query(
        "SELECT id,workflow_id AS workflowId,workflow_version AS workflowVersion,status,error,\
         created_at AS createdAt,updated_at AS updatedAt,completed_at AS completedAt \
         FROM spaces_creation_workflow_runs WHERE space_id=$1 ORDER BY created_at,id",
    )
    .bind(space_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let mut values = crate::sqlite::rows_to_json(rows)?;
    for value in &mut values {
        let id = value
            .get("id")
            .and_then(Value::as_str)
            .ok_or("Exported Creation Workflow run ID is unavailable")?;
        let rows = sqlx::query(
            "SELECT s.step_id AS id,s.action,s.status,(SELECT json_group_array(o.value) FROM json_each(s.output_node_ids) o JOIN spaces_nodes n ON n.id=o.value AND n.deleted_at IS NULL AND n.space_id=r.space_id) AS output_node_ids,s.error,s.created_at AS createdAt,\
             s.updated_at AS updatedAt,s.completed_at AS completedAt \
             FROM spaces_creation_workflow_steps s JOIN spaces_creation_workflow_runs r ON r.id=s.run_id WHERE s.run_id=$1 ORDER BY s.created_at,s.step_id",
        )
        .bind(id)
        .fetch_all(&mut **tx)
        .await
        .map_err(db::sql_error)?;
        let steps = rows.into_iter().map(|row| {
            let output: String = row.try_get("output_node_ids").map_err(db::sql_error)?;
            Ok(serde_json::json!({
                "id": row.try_get::<String, _>("id").map_err(db::sql_error)?,
                "action": row.try_get::<String, _>("action").map_err(db::sql_error)?,
                "status": row.try_get::<String, _>("status").map_err(db::sql_error)?,
                "outputNodeIds": serde_json::from_str::<Value>(&output).map_err(db::json_error)?,
                "error": row.try_get::<String, _>("error").map_err(db::sql_error)?,
                "createdAt": row.try_get::<i64, _>("createdAt").map_err(db::sql_error)?,
                "updatedAt": row.try_get::<i64, _>("updatedAt").map_err(db::sql_error)?,
                "completedAt": row.try_get::<Option<i64>, _>("completedAt").map_err(db::sql_error)?,
            }))
        }).collect::<Result<Vec<_>, String>>()?;
        value
            .as_object_mut()
            .ok_or("Exported Creation Workflow run must be an object")?
            .insert("steps".into(), Value::Array(steps));
    }
    Ok(values)
}
