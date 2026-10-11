use crate::desktop::tasks::contracts::{DesktopTask, DesktopTaskState};
use sqlx::{Row, SqlitePool};

pub(crate) async fn snapshot(pool: &SqlitePool) -> Result<Vec<DesktopTask>, String> {
    let cutoff = crate::foundation::clock::unix_time_millis()? - 600_000;
    let rows = sqlx::query("SELECT id,state,created_at FROM spaces_runs WHERE parent_run_id IS NULL AND (completed_at IS NULL OR completed_at>$1)")
        .bind(cutoff).fetch_all(pool).await.map_err(crate::spaces::db::sql_error)?;
    rows.iter()
        .map(|row| {
            let state: String = row.try_get("state").map_err(crate::spaces::db::sql_error)?;
            let state = match state.as_str() {
                "queued" => DesktopTaskState::Queued,
                "running" => DesktopTaskState::Running,
                "waiting" | "paused" => DesktopTaskState::Attention,
                "succeeded" => DesktopTaskState::Succeeded,
                "failed" | "completed_with_errors" => DesktopTaskState::Failed,
                "cancelled" => DesktopTaskState::Cancelled,
                _ => return Err("invalid durable Run task state".into()),
            };
            Ok(DesktopTask {
                id: row.try_get("id").map_err(crate::spaces::db::sql_error)?,
                state,
                progress: None,
                started_at: row
                    .try_get("created_at")
                    .map_err(crate::spaces::db::sql_error)?,
            })
        })
        .collect()
}
