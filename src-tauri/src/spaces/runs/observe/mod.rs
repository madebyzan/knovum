use crate::app_error::AppError;
use sqlx::SqlitePool;
use tauri::ipc::Channel;

use super::{load_run, run_rows, subscribe_to_changes, RunObservation};
use crate::spaces::db;

#[tauri::command]
pub async fn list_active_resource_runs(
    app: tauri::AppHandle,
) -> Result<Vec<RunObservation>, AppError> {
    let pool: &SqlitePool = db::pool(&app).await?;
    Ok(active_resource_runs(pool).await?)
}

async fn active_resource_runs(pool: &SqlitePool) -> Result<Vec<RunObservation>, String> {
    let rows = sqlx::query(
        "SELECT * FROM spaces_runs WHERE kind IN ('workflow','action')
         AND parent_run_id IS NULL AND completed_at IS NULL ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    rows.iter()
        .map(run_rows::parse_summary)
        .collect::<Result<_, _>>()
}

#[cfg(test)]
mod tests;

#[tauri::command]
pub async fn observe_run(
    app: tauri::AppHandle,
    run_id: String,
    updates: Channel<RunObservation>,
) -> Result<(), AppError> {
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
