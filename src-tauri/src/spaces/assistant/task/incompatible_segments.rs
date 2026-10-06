use tauri::AppHandle;

use crate::spaces::{
    assistant::{tools, AssistantMode},
    db, runs,
};

pub(super) async fn retire(app: &AppHandle, pool: &sqlx::SqlitePool) -> Result<(), String> {
    let records: Vec<String> = sqlx::query_scalar("SELECT r.id FROM spaces_assistant_tasks t JOIN spaces_runs r ON r.id=t.run_id
        JOIN spaces_assistant_execution_segments s ON s.id=t.segment_id WHERE r.completed_at IS NULL
        AND (s.graph_version!=$1 OR json_extract(r.policy_snapshot,'$.assistantProfileVersion') IS NOT $2)")
        .bind(crate::agent_runtime::protocol::GRAPH_VERSION).bind(i64::from(super::ASSISTANT_PROFILE_VERSION))
        .fetch_all(pool).await.map_err(db::sql_error)?;
    for run_id in records {
        cancel(
            app,
            pool,
            &run_id,
            "assistant-graph-version-changed: start a new execution segment",
        )
        .await?;
    }
    for mode in [
        AssistantMode::Ask,
        AssistantMode::Plan,
        AssistantMode::Execute,
    ] {
        let digest = tools::project(mode)?.digest;
        let incompatible: Vec<String> = sqlx::query_scalar("SELECT r.id FROM spaces_assistant_tasks t JOIN spaces_runs r ON r.id=t.run_id
            WHERE r.completed_at IS NULL AND r.cancel_requested_at IS NULL AND json_extract(r.policy_snapshot,'$.mode')=$1 AND t.catalog_digest!=$2")
            .bind(mode.as_str()).bind(digest).fetch_all(pool).await.map_err(db::sql_error)?;
        for run_id in incompatible {
            cancel(
                app,
                pool,
                &run_id,
                "assistant-action-catalog-changed: start a new execution segment",
            )
            .await?;
        }
    }
    Ok(())
}

async fn cancel(
    app: &AppHandle,
    pool: &sqlx::SqlitePool,
    run_id: &str,
    error: &str,
) -> Result<(), String> {
    sqlx::query("UPDATE spaces_assistant_tasks SET error=$1,updated_at=$2 WHERE run_id=$3")
        .bind(error)
        .bind(crate::clock::unix_time_millis()?)
        .bind(run_id)
        .execute(pool)
        .await
        .map_err(db::sql_error)?;
    runs::cancel_run(app.clone(), run_id.into())
        .await
        .map_err(|error| error.to_string())
}
