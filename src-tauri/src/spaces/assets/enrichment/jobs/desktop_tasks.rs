use crate::{
    desktop::tasks::contracts::{DesktopTask, DesktopTaskState},
    spaces::db,
};
use sqlx::{Row, SqlitePool};

pub(crate) async fn snapshot(pool: &SqlitePool) -> Result<Vec<DesktopTask>, String> {
    let cutoff = crate::foundation::clock::unix_time_millis()? - 600_000;
    let rows = sqlx::query("SELECT id,status,created_at FROM spaces_enrichment_jobs WHERE status IN ('pending','running','awaiting_configuration') OR updated_at>$1")
        .bind(cutoff).fetch_all(pool).await.map_err(db::sql_error)?;
    rows.iter()
        .map(|row| {
            let status: String = row.try_get("status").map_err(db::sql_error)?;
            Ok(DesktopTask {
                id: row.try_get("id").map_err(db::sql_error)?,
                state: match status.as_str() {
                    "pending" => DesktopTaskState::Queued,
                    "running" => DesktopTaskState::Running,
                    "awaiting_configuration" => DesktopTaskState::Attention,
                    "succeeded" => DesktopTaskState::Succeeded,
                    "failed" => DesktopTaskState::Failed,
                    "cancelled" => DesktopTaskState::Cancelled,
                    _ => return Err("invalid enrichment task state".into()),
                },
                progress: None,
                started_at: row.try_get("created_at").map_err(db::sql_error)?,
            })
        })
        .collect()
}
