use super::super::{
    contracts::{
        PluginInspection, PluginInstallation, PluginOperationStage, PluginPublication,
        PluginSource, PluginUpdatePreview,
    },
    database, ExtensionState,
};
use super::{installation, operations::PluginOperation, packages, sources};
use crate::app_error::AppResult;
use tauri::{ipc::Channel, AppHandle, Manager};

#[tauri::command]
pub async fn inspect_plugin_source(
    app: AppHandle,
    source: PluginSource,
    job_id: String,
    on_progress: Channel<PluginOperationStage>,
) -> AppResult<PluginInspection> {
    let operation = PluginOperation::begin(
        &job_id,
        on_progress,
        &app.state::<ExtensionState>().shutdown,
    )?;
    let (files, revision) = operation
        .read(sources::read(&source, operation.cancellation()))
        .await?;
    Ok(packages::inspect(&files, revision)?.inspection)
}

#[tauri::command]
pub async fn install_plugin(
    app: AppHandle,
    source: PluginSource,
    expected_digest: String,
    publication: Option<PluginPublication>,
    job_id: String,
    on_progress: Channel<PluginOperationStage>,
) -> AppResult<super::super::contracts::ExtensionChange<PluginInstallation>> {
    let operation = PluginOperation::begin(
        &job_id,
        on_progress,
        &app.state::<ExtensionState>().shutdown,
    )?;
    let (files, revision) = operation
        .read(sources::read(&source, operation.cancellation()))
        .await?;
    let reviewed = packages::inspect(&files, revision)?;
    if reviewed.inspection.digest != expected_digest {
        return Err("plugin-source-changed-review-again".into());
    }
    let state = app.state::<ExtensionState>();
    let _guard = state.content_lock.lock().await;
    if let Some(publication) = &publication {
        super::marketplaces::require_publication(&app, publication, &source).await?;
        if reviewed.inspection.manifest.name != publication.plugin_name {
            return Err("marketplace-package-name-mismatch".into());
        }
    }
    operation.commit()?;
    Ok(installation::commit(
        &app,
        &uuid::Uuid::new_v4().to_string(),
        source,
        reviewed,
        files,
        false,
        publication,
    )
    .await?)
}

#[tauri::command]
pub async fn update_plugin(
    app: AppHandle,
    id: String,
    expected_digest: String,
    job_id: String,
    on_progress: Channel<PluginOperationStage>,
) -> AppResult<super::super::contracts::ExtensionChange<PluginInstallation>> {
    let operation = PluginOperation::begin(
        &job_id,
        on_progress,
        &app.state::<ExtensionState>().shutdown,
    )?;
    let old = super::load(&database::pool(&app).await?, &id).await?;
    if old.builtin {
        return Err("builtin-plugin-updated-with-application".into());
    }
    let source = super::marketplaces::update_source(&app, &old).await?;
    let (files, revision) = operation
        .read(sources::read(&source, operation.cancellation()))
        .await?;
    let reviewed = packages::inspect(&files, revision)?;
    if reviewed.inspection.digest != expected_digest {
        return Err("plugin-source-changed-review-again".into());
    }
    if reviewed.inspection.manifest.name != old.revision.manifest.name {
        return Err("plugin-update-identity-changed".into());
    }
    let state = app.state::<ExtensionState>();
    let _guard = state.content_lock.lock().await;
    let current = super::load(&database::pool(&app).await?, &id).await?;
    if current.revision.digest != old.revision.digest {
        return Err("plugin-updated-concurrently-review-again".into());
    }
    if let Some(publication) = &old.publication {
        super::marketplaces::require_publication(&app, publication, &source).await?;
    }
    operation.commit()?;
    Ok(installation::commit(&app, &id, source, reviewed, files, false, old.publication).await?)
}

#[tauri::command]
pub async fn list_plugin_installations(app: AppHandle) -> Result<Vec<PluginInstallation>, String> {
    super::ensure_builtin(&app).await?;
    super::list(&database::pool(&app).await?).await
}

#[tauri::command]
pub async fn set_plugin_enabled(app: AppHandle, id: String, enabled: bool) -> Result<(), String> {
    let state = app.state::<ExtensionState>();
    let _guard = state.content_lock.lock().await;
    let pool = database::pool(&app).await?;
    let mut plugin = super::load(&pool, &id).await?;
    plugin.enabled = enabled;
    sqlx::query("UPDATE plugins SET document=$1 WHERE id=$2")
        .bind(database::encode(&plugin)?)
        .bind(&id)
        .execute(&pool)
        .await
        .map_err(|error| error.to_string())?;
    if !enabled {
        super::super::mcp::sessions::revoke_plugin(&app, &id).await;
    }
    Ok(())
}

#[tauri::command]
pub async fn uninstall_plugin(
    app: AppHandle,
    id: String,
) -> Result<super::super::contracts::ExtensionChange<()>, String> {
    let state = app.state::<ExtensionState>();
    let _guard = state.content_lock.lock().await;
    let pool = database::pool(&app).await?;
    if super::load(&pool, &id).await?.builtin {
        return Err("builtin-plugin-can-be-disabled".into());
    }
    let mut tx = pool.begin().await.map_err(|error| error.to_string())?;
    sqlx::query("DELETE FROM plugins WHERE id=$1")
        .bind(&id)
        .execute(&mut *tx)
        .await
        .map_err(|error| error.to_string())?;
    sqlx::query("DELETE FROM skills WHERE json_extract(document,'$.pluginId')=$1")
        .bind(&id)
        .execute(&mut *tx)
        .await
        .map_err(|error| error.to_string())?;
    sqlx::query("DELETE FROM connections WHERE json_extract(document,'$.connectorId') IN (SELECT id FROM connectors WHERE json_extract(document,'$.pluginId')=$1)")
        .bind(&id)
        .execute(&mut *tx)
        .await
        .map_err(|error| error.to_string())?;
    sqlx::query("DELETE FROM connectors WHERE json_extract(document,'$.pluginId')=$1")
        .bind(&id)
        .execute(&mut *tx)
        .await
        .map_err(|error| error.to_string())?;
    tx.commit().await.map_err(|error| error.to_string())?;
    super::super::mcp::sessions::revoke_plugin(&app, &id).await;
    Ok(super::super::contracts::ExtensionChange::committed(
        (),
        [
            super::super::connectors::cleanup::collect(&app).await,
            super::super::reclamation::collect(&app).await,
        ],
    ))
}

#[tauri::command]
pub async fn inspect_plugin_update(
    app: AppHandle,
    id: String,
    job_id: String,
    on_progress: Channel<PluginOperationStage>,
) -> AppResult<PluginUpdatePreview> {
    let operation = PluginOperation::begin(
        &job_id,
        on_progress,
        &app.state::<ExtensionState>().shutdown,
    )?;
    let plugin = super::load(&database::pool(&app).await?, &id).await?;
    if plugin.builtin {
        return Err("builtin-plugin-updated-with-application".into());
    }
    let source = super::marketplaces::update_source(&app, &plugin).await?;
    let (files, revision) = operation
        .read(sources::read(&source, operation.cancellation()))
        .await?;
    let inspection = packages::inspect(&files, revision)?.inspection;
    Ok(PluginUpdatePreview { source, inspection })
}
