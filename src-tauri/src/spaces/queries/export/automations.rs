use serde_json::{json, Value};
use sqlx::{Row, Sqlite, Transaction};

use crate::spaces::db;

pub(super) async fn rows(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
) -> Result<(Vec<Value>, Vec<Value>), String> {
    let triggers = sqlx::query(
        "SELECT * FROM spaces_automation_triggers WHERE space_id=$1 ORDER BY created_at,id",
    )
    .bind(space_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let trigger_rows = triggers.iter().map(|row| Ok(json!({
        "id": text(row,"id")?, "name": text(row,"name")?, "workflowId": text(row,"workflow_id")?,
        "workflowVersion": integer(row,"workflow_version")?, "source": parsed(row,"source_config")?,
        "input": parsed(row,"input")?, "overlapPolicy": text(row,"overlap_policy")?,
        "createdAt": integer(row,"created_at")?, "updatedAt": integer(row,"updated_at")?,
    }))).collect::<Result<Vec<_>, String>>()?;
    let occurrences = sqlx::query("SELECT o.* FROM spaces_automation_occurrences o JOIN spaces_automation_triggers t ON t.id=o.trigger_id WHERE t.space_id=$1 ORDER BY o.created_at,o.id")
        .bind(space_id).fetch_all(&mut **tx).await.map_err(db::sql_error)?;
    let occurrence_rows = occurrences.iter().map(|row| Ok(json!({
        "id": text(row,"id")?, "triggerId": text(row,"trigger_id")?,
        "scheduledAt": integer(row,"scheduled_at")?, "occurrenceKey": text(row,"occurrence_key")?,
        "runId": optional_text(row,"run_id")?, "state": text(row,"state")?,
        "detail": parsed(row,"detail")?, "createdAt": integer(row,"created_at")?,
    }))).collect::<Result<Vec<_>, String>>()?;
    Ok((trigger_rows, occurrence_rows))
}

fn text(row: &sqlx::sqlite::SqliteRow, key: &str) -> Result<String, String> {
    row.try_get(key).map_err(db::sql_error)
}
fn optional_text(row: &sqlx::sqlite::SqliteRow, key: &str) -> Result<Option<String>, String> {
    row.try_get(key).map_err(db::sql_error)
}
fn integer(row: &sqlx::sqlite::SqliteRow, key: &str) -> Result<i64, String> {
    row.try_get(key).map_err(db::sql_error)
}
fn parsed(row: &sqlx::sqlite::SqliteRow, key: &str) -> Result<Value, String> {
    serde_json::from_str(&text(row, key)?).map_err(db::json_error)
}
