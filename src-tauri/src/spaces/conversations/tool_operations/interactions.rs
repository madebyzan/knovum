use crate::spaces::{db, runs};
use futures::future::BoxFuture;
use rmcp::model::{ElicitRequestParams, ElicitResult, ProgressNotificationParam};
use serde_json::Value;
use sqlx::SqlitePool;
use tauri::AppHandle;

pub(crate) struct RunMcpInteraction {
    pub pool: SqlitePool,
    pub run_id: String,
}

impl crate::extensions::mcp::McpInteraction for RunMcpInteraction {
    fn elicit(
        &self,
        request: ElicitRequestParams,
        cancellation: tokio_util::sync::CancellationToken,
    ) -> BoxFuture<'_, Result<ElicitResult, String>> {
        Box::pin(async move {
            let id = uuid::Uuid::new_v4().to_string();
            let now = crate::clock::unix_time_millis()?;
            let mut changes = runs::subscribe_to_changes();
            let mut tx = db::begin_write(&self.pool).await?;
            sqlx::query("INSERT INTO spaces_conversation_mcp_interactions(id,run_id,request,created_at) VALUES($1,$2,$3,$4)").bind(&id).bind(&self.run_id).bind(serde_json::to_string(&request).map_err(db::json_error)?).bind(now).execute(&mut *tx).await.map_err(db::sql_error)?;
            let waiting = sqlx::query("UPDATE spaces_runs SET state='waiting',wait_reason='mcp_elicitation',state_version=state_version+1,updated_at=$1 WHERE id=$2 AND completed_at IS NULL AND cancel_requested_at IS NULL AND pause_requested_at IS NULL")
                .bind(now).bind(&self.run_id).execute(&mut *tx).await.map_err(db::sql_error)?;
            db::require_one(waiting, "conversation-tool-run-inactive")?;
            tx.commit().await.map_err(db::sql_error)?;
            runs::announce_change();
            loop {
                let run = runs::load_run(&self.pool, &self.run_id).await?;
                if run.state.is_terminal()
                    || run.state == runs::RunState::Paused
                    || cancellation.is_cancelled()
                {
                    sqlx::query("UPDATE spaces_conversation_mcp_interactions SET response='{\"action\":\"cancel\"}' WHERE id=$1 AND response IS NULL").bind(&id).execute(&self.pool).await.map_err(db::sql_error)?;
                    clear_wait(&self.pool, &self.run_id).await?;
                    runs::announce_change();
                    return Ok(ElicitResult::new(rmcp::model::ElicitationAction::Cancel));
                }
                let response: Option<String> = sqlx::query_scalar(
                    "SELECT response FROM spaces_conversation_mcp_interactions WHERE id=$1",
                )
                .bind(&id)
                .fetch_one(&self.pool)
                .await
                .map_err(db::sql_error)?;
                if let Some(response) = response {
                    return serde_json::from_str(&response).map_err(db::json_error);
                }
                tokio::select! { _ = cancellation.cancelled() => {}, result = changes.changed() => { result.map_err(|_| "run-observation-closed")?; } }
            }
        })
    }
    fn progress(&self, progress: ProgressNotificationParam) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            let result = async {
                let mut tx = db::begin_write(&self.pool).await?;
                runs::append_event(
                    &mut tx,
                    &self.run_id,
                    None,
                    None,
                    runs::RunEventKind::McpProgress,
                    &serde_json::to_value(progress).map_err(db::json_error)?,
                    crate::clock::unix_time_millis()?,
                )
                .await?;
                tx.commit().await.map_err(db::sql_error)?;
                runs::announce_change();
                Ok::<_, String>(())
            }
            .await;
            if let Err(error) = result {
                log::error!("MCP progress recording failed: {error}");
            }
        })
    }
}

#[derive(serde::Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct McpUserInteraction {
    pub id: String,
    pub request: Value,
}

#[tauri::command]
pub async fn list_mcp_user_interactions(
    app: AppHandle,
    run_id: String,
) -> Result<Vec<McpUserInteraction>, String> {
    let rows: Vec<(String,String)> = sqlx::query_as("SELECT i.id,i.request FROM spaces_conversation_mcp_interactions i JOIN spaces_runs r ON r.id=i.run_id WHERE i.run_id=$1 AND i.response IS NULL AND r.completed_at IS NULL AND r.wait_reason='mcp_elicitation' ORDER BY i.created_at").bind(run_id).fetch_all(db::pool(&app).await?).await.map_err(db::sql_error)?;
    rows.into_iter()
        .map(|(id, request)| {
            Ok(McpUserInteraction {
                id,
                request: serde_json::from_str(&request).map_err(db::json_error)?,
            })
        })
        .collect()
}

#[tauri::command]
pub async fn answer_mcp_user_interaction(
    app: AppHandle,
    id: String,
    response: Value,
) -> Result<(), String> {
    let response: ElicitResult = serde_json::from_value(response).map_err(db::json_error)?;
    let pool = db::pool(&app).await?;
    let row: (String,String) = sqlx::query_as("SELECT run_id,request FROM spaces_conversation_mcp_interactions WHERE id=$1 AND response IS NULL").bind(&id).fetch_one(pool).await.map_err(db::sql_error)?;
    let request: ElicitRequestParams = serde_json::from_str(&row.1).map_err(db::json_error)?;
    if response.action == rmcp::model::ElicitationAction::Accept {
        if let ElicitRequestParams::FormElicitationParams {
            requested_schema, ..
        } = request
        {
            let schema = serde_json::to_value(requested_schema).map_err(db::json_error)?;
            jsonschema::validator_for(&schema)
                .map_err(|error| error.to_string())?
                .validate(
                    response
                        .content
                        .as_ref()
                        .ok_or("mcp-elicitation-form-content-required")?,
                )
                .map_err(|error| error.to_string())?;
        }
    }
    let mut tx = db::begin_write(pool).await?;
    let updated = sqlx::query("UPDATE spaces_conversation_mcp_interactions SET response=$1 WHERE id=$2 AND response IS NULL AND EXISTS(SELECT 1 FROM spaces_runs r WHERE r.id=run_id AND r.completed_at IS NULL AND r.cancel_requested_at IS NULL AND r.pause_requested_at IS NULL)").bind(serde_json::to_string(&response).map_err(db::json_error)?).bind(id).execute(&mut *tx).await.map_err(db::sql_error)?;
    db::require_one(updated, "mcp-elicitation-response-stale")?;
    sqlx::query("UPDATE spaces_runs SET state='running',wait_reason=NULL,state_version=state_version+1 WHERE id=$1 AND state='waiting' AND wait_reason='mcp_elicitation' AND completed_at IS NULL AND cancel_requested_at IS NULL AND pause_requested_at IS NULL AND NOT EXISTS(SELECT 1 FROM spaces_conversation_mcp_interactions WHERE run_id=$1 AND response IS NULL)").bind(row.0).execute(&mut *tx).await.map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)?;
    runs::announce_change();
    Ok(())
}

async fn clear_wait(pool: &SqlitePool, run_id: &str) -> Result<(), String> {
    sqlx::query("UPDATE spaces_runs SET state='running',wait_reason=NULL,state_version=state_version+1 WHERE id=$1 AND state='waiting' AND wait_reason='mcp_elicitation' AND completed_at IS NULL AND cancel_requested_at IS NULL AND pause_requested_at IS NULL AND NOT EXISTS(SELECT 1 FROM spaces_conversation_mcp_interactions WHERE run_id=$1 AND response IS NULL)").bind(run_id).execute(pool).await.map_err(db::sql_error)?;
    Ok(())
}
