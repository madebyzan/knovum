#[tauri::command]
pub async fn reconcile_workspace(app: tauri::AppHandle) -> Result<(), String> {
    let pool = super::db::pool(&app).await?;
    super::knowledge::retrieval::invalidation_outbox::drain(&app, pool).await?;
    super::assets::reconcile_interrupted_work(&app).await?;
    super::agent_artifacts::recovery::discard_orphaned_workspaces(&app).await?;
    super::derivation::reconcile_interrupted(&app).await?;
    super::assets::enrichment::jobs::reconcile_interrupted(&app).await
}
