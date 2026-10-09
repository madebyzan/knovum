use crate::spaces::db;

pub(crate) async fn reconcile_terminal_snapshots<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    pool: &sqlx::SqlitePool,
) -> Result<(), String> {
    cleanup(app, pool, None).await
}

pub(crate) async fn cleanup_terminal_snapshot<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    pool: &sqlx::SqlitePool,
    run_id: &str,
) -> Result<(), String> {
    cleanup(app, pool, Some(run_id)).await
}

async fn cleanup<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    pool: &sqlx::SqlitePool,
    run_id: Option<&str>,
) -> Result<(), String> {
    // Terminal Runs cannot resume source inference. Even a published candidate may leave
    // this operational snapshot when its post-commit file removal was interrupted.
    let snapshots: Vec<(String, String)> = sqlx::query_as(
        "SELECT terminal.id,json_extract(terminal.input_snapshot,'$.referenceSource.assetPath') \
         FROM spaces_runs terminal \
         WHERE terminal.scope_kind='application' AND terminal.completed_at IS NOT NULL \
           AND (? IS NULL OR terminal.id=?) \
           AND json_extract(terminal.input_snapshot,'$.workflow')=? \
           AND json_type(terminal.input_snapshot,'$.referenceSource.assetPath')='text' \
           AND NOT EXISTS(SELECT 1 FROM spaces_runs active \
             WHERE active.scope_kind='application' AND active.completed_at IS NULL \
               AND json_extract(active.input_snapshot,'$.workflow')=? \
               AND json_extract(active.input_snapshot,'$.referenceSource.assetPath')= \
                   json_extract(terminal.input_snapshot,'$.referenceSource.assetPath')) \
         ORDER BY terminal.completed_at,terminal.id",
    )
    .bind(run_id)
    .bind(run_id)
    .bind(super::WORKFLOW)
    .bind(super::WORKFLOW)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    for (run_id, asset_path) in snapshots {
        super::source_snapshot::remove_managed_snapshot(app, &run_id, &asset_path)?;
    }
    Ok(())
}
