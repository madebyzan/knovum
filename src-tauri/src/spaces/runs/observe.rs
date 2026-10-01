use crate::command_error::CommandError;
use sqlx::SqlitePool;
use tauri::ipc::Channel;

use super::{load_run, run_rows, subscribe_to_changes, RunObservation};
use crate::spaces::db;

#[tauri::command]
pub async fn list_active_resource_runs(
    app: tauri::AppHandle,
) -> Result<Vec<RunObservation>, CommandError> {
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
mod tests {
    use super::*;

    #[tokio::test]
    async fn task_center_restores_roots_without_duplicate_child_actions() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        db::migrate(&pool).await.unwrap();
        sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)")
            .execute(&pool).await.unwrap();
        for (id, kind, parent, completed) in [
            ("workflow", "workflow", None, None),
            ("action", "action", None, None),
            ("child", "action", Some("workflow"), None),
            ("assistant", "assistant", None, None),
            ("completed", "action", None, Some(2_i64)),
        ] {
            sqlx::query("INSERT INTO spaces_runs(id,home_space_id,parent_run_id,principal_kind,principal_id,kind,state,policy_snapshot,input_snapshot,budget,created_at,updated_at,completed_at) VALUES($1,'space',$2,'user','local-user',$3,$4,'{}','{}','{}',1,1,$5)")
                .bind(id).bind(parent).bind(kind)
                .bind(if completed.is_some() { "succeeded" } else { "running" })
                .bind(completed).execute(&pool).await.unwrap();
        }
        let mut ids = active_resource_runs(&pool)
            .await
            .unwrap()
            .into_iter()
            .map(|run| run.id)
            .collect::<Vec<_>>();
        ids.sort();
        assert_eq!(ids, ["action", "workflow"]);
    }
}

#[tauri::command]
pub async fn observe_run(
    app: tauri::AppHandle,
    run_id: String,
    updates: Channel<RunObservation>,
) -> Result<(), CommandError> {
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
