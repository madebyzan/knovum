use sqlx::SqlitePool;
use tauri::ipc::Channel;

use super::{load_run, run_rows, subscribe_to_changes, RunSummary};
use crate::spaces::db;

#[tauri::command]
pub async fn list_active_workflow_runs(app: tauri::AppHandle) -> Result<Vec<RunSummary>, String> {
    let pool: &SqlitePool = db::pool(&app).await?;
    let rows = sqlx::query(
        "SELECT * FROM spaces_runs WHERE kind='workflow' AND completed_at IS NULL ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    rows.iter().map(run_rows::parse_summary).collect()
}

#[tauri::command]
pub async fn observe_run(
    app: tauri::AppHandle,
    run_id: String,
    updates: Channel<RunSummary>,
) -> Result<(), String> {
    let pool = crate::spaces::db::pool(&app).await?;
    let mut changes = subscribe_to_changes();
    loop {
        let run = load_run(pool, &run_id).await?;
        updates
            .send(run.clone())
            .map_err(|error| error.to_string())?;
        if run.state.is_terminal() {
            return Ok(());
        }
        changes.changed().await.map_err(|error| error.to_string())?;
    }
}
