#[tauri::command]
pub async fn reconcile_workspace(app: tauri::AppHandle) -> Result<(), String> {
    let pool = super::db::pool(&app).await?;
    let mut failures = Vec::new();

    collect_recovery(
        &mut failures,
        "retrieval invalidation",
        super::knowledge::retrieval::invalidation_outbox::drain(&app, pool).await,
    );
    collect_recovery(
        &mut failures,
        "asset recovery",
        super::assets::reconcile_interrupted_work(&app).await,
    );
    collect_recovery(
        &mut failures,
        "agent workspace cleanup",
        super::agent_artifacts::recovery::discard_orphaned_workspaces(&app).await,
    );
    collect_recovery(
        &mut failures,
        "derivation recovery",
        super::derivation::reconcile_interrupted(&app).await,
    );
    collect_recovery(
        &mut failures,
        "enrichment job recovery",
        super::assets::enrichment::jobs::reconcile_interrupted(&app).await,
    );

    if failures.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "Workspace recovery failed: {}",
            failures.join("; ")
        ))
    }
}

fn collect_recovery(failures: &mut Vec<String>, owner: &str, result: Result<(), String>) {
    if let Err(error) = result {
        failures.push(format!("{owner}: {error}"));
    }
}
