use crate::spaces::db;

pub(in crate::spaces) async fn release_run(
    app: &tauri::AppHandle,
    pool: &sqlx::SqlitePool,
    run_id: &str,
) -> Result<(), String> {
    let snapshot: Option<String> = sqlx::query_scalar(
        "SELECT json_extract(input_snapshot,'$.extensions.id') FROM spaces_runs WHERE id=$1",
    )
    .bind(run_id)
    .fetch_one(pool)
    .await
    .map_err(db::sql_error)?;
    if let Some(snapshot) = snapshot {
        crate::extensions::reclamation::release(app, &snapshot).await?;
    }
    Ok(())
}
