use serde_json::{json, Value};
use sqlx::SqlitePool;

use crate::spaces::db;

pub(in crate::spaces::conversations) async fn observe(
    pool: &SqlitePool,
    child_run_id: &str,
) -> Result<Option<Value>, String> {
    let run = crate::spaces::runs::load_run(pool, child_run_id).await?;
    if !run.state.is_terminal() {
        return Ok(None);
    }
    let output = sqlx::query_scalar::<_, Option<String>>(
        "SELECT output FROM spaces_run_steps WHERE run_id=$1 ORDER BY updated_at DESC LIMIT 1",
    )
    .bind(child_run_id)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?
    .flatten()
    .map(|value| serde_json::from_str::<Value>(&value).map_err(db::json_error))
    .transpose()?;
    let node_id: Option<String> = sqlx::query_scalar("SELECT output_node_id FROM spaces_knowledge_derivations WHERE id=$1 AND status='completed'")
        .bind(child_run_id).fetch_optional(pool).await.map_err(db::sql_error)?;
    Ok(Some(
        json!({"type":"accepted_run","runId":child_run_id,"state":run.state,"output":output,"nodeId":node_id}),
    ))
}

pub(in crate::spaces::conversations) async fn wait(
    pool: &SqlitePool,
    child_run_id: &str,
) -> Result<(), String> {
    let mut changes = crate::spaces::runs::subscribe_to_changes();
    loop {
        if observe(pool, child_run_id).await?.is_some() {
            return Ok(());
        }
        changes
            .changed()
            .await
            .map_err(|_| "run-change-signal-closed")?;
    }
}
