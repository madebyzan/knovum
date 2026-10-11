use super::super::{
    contracts::{
        PluginInspection, PluginInstallation, PluginOperationStage, PluginPublication,
        PluginSource, PluginUpdatePreview,
    },
    database, ExtensionState,
};
use super::{installation, operations::PluginOperation, packages, sources};
use crate::foundation::app_error::AppResult;
use tauri::{ipc::Channel, AppHandle, Manager};

#[tauri::command]
pub async fn inspect_plugin_source(
    app: AppHandle,
    source: PluginSource,
    job_id: String,
    on_stage: Channel<PluginOperationStage>,
) -> AppResult<PluginInspection> {
    let operation =
        PluginOperation::begin(&job_id, on_stage, &app.state::<ExtensionState>().shutdown)?;
    let (files, revision) = operation
        .read(sources::read(&app, &source, operation.cancellation()))
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
    on_stage: Channel<PluginOperationStage>,
) -> AppResult<super::super::contracts::ExtensionChange<PluginInstallation>> {
    let operation =
        PluginOperation::begin(&job_id, on_stage, &app.state::<ExtensionState>().shutdown)?;
    let (files, revision) = operation
        .read(sources::read(&app, &source, operation.cancellation()))
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
    let installed = super::list(&database::pool(&app).await?).await?;
    if publication.as_ref().is_some_and(|publication| {
        installed.iter().any(|plugin| {
            plugin.publication.as_ref().is_some_and(|existing| {
                existing.marketplace_id == publication.marketplace_id
                    && existing.publisher_id == publication.publisher_id
                    && existing.plugin_name == publication.plugin_name
            })
        })
    }) {
        return Err("marketplace-plugin-already-installed".into());
    }
    let (plugin_id, is_bundled) = match &source {
        PluginSource::Bundled { name } => {
            let package = super::bundled::require(name)?;
            if installed.iter().any(|plugin| plugin.id == package.id) {
                return Err("bundled-plugin-already-installed".into());
            }
            (package.id.to_owned(), true)
        }
        _ => (uuid::Uuid::new_v4().to_string(), false),
    };
    operation.commit()?;
    Ok(installation::commit(
        &app,
        &plugin_id,
        source,
        reviewed,
        files,
        is_bundled,
        publication,
    )
    .await?)
}

#[tauri::command]
pub async fn update_plugin(
    app: AppHandle,
    plugin_id: String,
    expected_digest: String,
    job_id: String,
    on_stage: Channel<PluginOperationStage>,
) -> AppResult<super::super::contracts::ExtensionChange<PluginInstallation>> {
    let operation =
        PluginOperation::begin(&job_id, on_stage, &app.state::<ExtensionState>().shutdown)?;
    let old = super::load(&database::pool(&app).await?, &plugin_id).await?;
    if old.builtin {
        return Err("builtin-plugin-updated-with-application".into());
    }
    let source = super::marketplaces::update_source(&app, &old).await?;
    let (files, revision) = operation
        .read(sources::read(&app, &source, operation.cancellation()))
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
    let current = super::load(&database::pool(&app).await?, &plugin_id).await?;
    if current.revision.digest != old.revision.digest {
        return Err("plugin-updated-concurrently-review-again".into());
    }
    if let Some(publication) = &old.publication {
        super::marketplaces::require_publication(&app, publication, &source).await?;
    }
    operation.commit()?;
    Ok(installation::commit(
        &app,
        &plugin_id,
        source,
        reviewed,
        files,
        false,
        old.publication,
    )
    .await?)
}

#[tauri::command]
pub async fn list_plugin_installations(app: AppHandle) -> Result<Vec<PluginInstallation>, String> {
    super::reconcile_bundled_plugins(&app).await?;
    super::list(&database::pool(&app).await?).await
}

#[tauri::command]
pub async fn enable_plugin(app: AppHandle, plugin_id: String) -> Result<(), String> {
    set_enabled(app, plugin_id, true).await
}

#[tauri::command]
pub async fn disable_plugin(app: AppHandle, plugin_id: String) -> Result<(), String> {
    set_enabled(app, plugin_id, false).await
}

async fn set_enabled(app: AppHandle, plugin_id: String, enabled: bool) -> Result<(), String> {
    let state = app.state::<ExtensionState>();
    let _guard = state.content_lock.lock().await;
    let pool = database::pool(&app).await?;
    let mut plugin = super::load(&pool, &plugin_id).await?;
    plugin.enabled = enabled;
    sqlx::query("UPDATE plugins SET document=$1 WHERE id=$2")
        .bind(sqlx::types::Json(&plugin))
        .bind(&plugin_id)
        .execute(&pool)
        .await
        .map_err(|error| error.to_string())?;
    if !enabled {
        super::super::mcp::sessions::revoke_plugin(&app, &plugin_id).await;
    }
    Ok(())
}

#[tauri::command]
pub async fn uninstall_plugin(
    app: AppHandle,
    plugin_id: String,
) -> Result<super::super::contracts::ExtensionChange<()>, String> {
    let state = app.state::<ExtensionState>();
    let _guard = state.content_lock.lock().await;
    let pool = database::pool(&app).await?;
    super::load(&pool, &plugin_id).await?;
    let mut tx = pool.begin().await.map_err(|error| error.to_string())?;
    sqlx::query("DELETE FROM plugins WHERE id=$1")
        .bind(&plugin_id)
        .execute(&mut *tx)
        .await
        .map_err(|error| error.to_string())?;
    sqlx::query("DELETE FROM skills WHERE json_extract(document,'$.pluginId')=$1")
        .bind(&plugin_id)
        .execute(&mut *tx)
        .await
        .map_err(|error| error.to_string())?;
    sqlx::query("DELETE FROM connections WHERE json_extract(document,'$.connectorId') IN (SELECT id FROM connectors WHERE json_extract(document,'$.pluginId')=$1)")
        .bind(&plugin_id)
        .execute(&mut *tx)
        .await
        .map_err(|error| error.to_string())?;
    sqlx::query("DELETE FROM connectors WHERE json_extract(document,'$.pluginId')=$1")
        .bind(&plugin_id)
        .execute(&mut *tx)
        .await
        .map_err(|error| error.to_string())?;
    tx.commit().await.map_err(|error| error.to_string())?;
    super::super::mcp::sessions::revoke_plugin(&app, &plugin_id).await;
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
    plugin_id: String,
    job_id: String,
    on_stage: Channel<PluginOperationStage>,
) -> AppResult<PluginUpdatePreview> {
    let operation =
        PluginOperation::begin(&job_id, on_stage, &app.state::<ExtensionState>().shutdown)?;
    let plugin = super::load(&database::pool(&app).await?, &plugin_id).await?;
    if plugin.builtin {
        return Err("builtin-plugin-updated-with-application".into());
    }
    let source = super::marketplaces::update_source(&app, &plugin).await?;
    let (files, revision) = operation
        .read(sources::read(&app, &source, operation.cancellation()))
        .await?;
    let inspection = packages::inspect(&files, revision)?.inspection;
    Ok(PluginUpdatePreview { source, inspection })
}
