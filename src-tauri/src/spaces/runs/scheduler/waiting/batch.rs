use serde_json::Value;
use sqlx::SqlitePool;

use crate::spaces::db;

pub(super) async fn settlement(
    pool: &SqlitePool,
    checkpoint: &Value,
    children: &[Value],
) -> Result<Option<Result<Value, String>>, String> {
    let mut results = checkpoint
        .get("completedItems")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut errors = Vec::new();
    for child in children {
        let key = child
            .get("itemKey")
            .and_then(Value::as_str)
            .ok_or("foreach-checkpoint-invalid: itemKey is missing")?;
        let id = child
            .get("childRunId")
            .and_then(Value::as_str)
            .ok_or("foreach-checkpoint-invalid: childRunId is missing")?;
        let child_checkpoint = serde_json::json!({"childRunId": id});
        let Some(settlement) =
            super::single_child_settlement(pool, Some(&child_checkpoint)).await?
        else {
            return Ok(None);
        };
        match settlement {
            Ok(value) => {
                settle_item(pool, id, "succeeded", Some(&value), None).await?;
                results.push(serde_json::json!({"itemKey": key, "result": value}));
            }
            Err(error) => {
                settle_item(pool, id, "failed", None, Some(&error)).await?;
                errors.push(serde_json::json!({"itemKey": key, "error": error}));
            }
        }
    }
    if !errors.is_empty()
        && checkpoint.get("errorPolicy").and_then(Value::as_str) != Some("collect_errors")
    {
        return Ok(Some(Err("foreach-item-failed".into())));
    }
    Ok(Some(Ok(
        serde_json::json!({"items": results, "errors": errors}),
    )))
}

async fn settle_item(
    pool: &SqlitePool,
    child_run_id: &str,
    state: &str,
    output: Option<&Value>,
    error: Option<&str>,
) -> Result<(), String> {
    let output = output
        .map(serde_json::to_string)
        .transpose()
        .map_err(db::json_error)?;
    sqlx::query("UPDATE spaces_run_batch_items SET state=$1,output=$2,error=$3,updated_at=$4 WHERE child_run_id=$5")
        .bind(state).bind(output).bind(error).bind(crate::clock::unix_time_millis()?)
        .bind(child_run_id).execute(pool).await.map_err(db::sql_error)?;
    Ok(())
}
