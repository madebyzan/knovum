use super::super::{
    contracts::{
        MarketplaceSource, PluginMarketplace, PluginOperationStage, PluginPublication,
        PluginRelease, PluginSource,
    },
    database, ExtensionState,
};
use super::operations::PluginOperation;
use crate::foundation::app_error::{AppError, AppResult};
use serde::Deserialize;
use tauri::{ipc::Channel, AppHandle, Manager};
use tokio_util::sync::CancellationToken;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct MarketplaceManifest {
    schema_version: u16,
    name: String,
    display_name: String,
    releases: Vec<PluginRelease>,
}

async fn read(
    app: &AppHandle,
    source: &MarketplaceSource,
    cancellation: &CancellationToken,
) -> AppResult<MarketplaceManifest> {
    let bytes = match source {
        MarketplaceSource::Bundled => {
            return Err("bundled-marketplace-is-application-managed".into())
        }
        MarketplaceSource::Directory { path } => {
            let path = std::path::PathBuf::from(path).join("marketplace.json");
            crate::foundation::blocking::run(move || {
                super::sources::read_bounded(&path, 4 * 1024 * 1024)
            })
            .await?
        }
        MarketplaceSource::Https { url } => super::sources::download(url, 4 * 1024 * 1024).await?,
        MarketplaceSource::Git {
            url,
            reference,
            subdirectory,
        } => {
            let (files, _) = super::sources::read(
                app,
                &PluginSource::Git {
                    url: url.clone(),
                    reference: reference.clone(),
                    subdirectory: subdirectory.clone(),
                },
                cancellation,
            )
            .await?;
            files
                .get("marketplace.json")
                .ok_or("marketplace-manifest-required")?
                .clone()
        }
    };
    let mut manifest: MarketplaceManifest =
        serde_json::from_slice(&bytes).map_err(|error| format!("marketplace-invalid: {error}"))?;
    if manifest.schema_version != 1
        || manifest.display_name.trim().is_empty()
        || manifest.name.trim().is_empty()
    {
        return Err("marketplace-v1-identity-required".into());
    }
    let mut identities = std::collections::BTreeSet::new();
    for release in &mut manifest.releases {
        if release.publisher_id.is_empty()
            || release.name.is_empty()
            || !identities.insert((&release.publisher_id, &release.name))
        {
            return Err("marketplace-release-identity-invalid".into());
        }
        match &mut release.source {
            PluginSource::HttpsZip {
                sha256: Some(digest),
                ..
            } if digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit()) => {}
            PluginSource::Git { url, reference, .. }
                if url.starts_with("https://") && !reference.is_empty() => {}
            PluginSource::Directory { path } | PluginSource::Zip { path } => {
                let MarketplaceSource::Directory { path: root } = source else {
                    return Err("remote-marketplace-cannot-reference-local-files".into());
                };
                super::super::content::validate_path(path)?;
                let root = std::fs::canonicalize(root).map_err(|error| error.to_string())?;
                let candidate =
                    std::fs::canonicalize(root.join(&*path)).map_err(|error| error.to_string())?;
                if !candidate.starts_with(&root) {
                    return Err("marketplace-release-path-escapes-root".into());
                }
                *path = candidate
                    .to_str()
                    .ok_or("marketplace-path-must-be-utf8")?
                    .into();
            }
            _ => return Err("marketplace-release-requires-pinned-zip-or-git".into()),
        }
    }
    Ok(manifest)
}

#[tauri::command]
pub async fn add_plugin_marketplace(
    app: AppHandle,
    source: MarketplaceSource,
    job_id: String,
    on_stage: Channel<PluginOperationStage>,
) -> AppResult<PluginMarketplace> {
    let operation =
        PluginOperation::begin(&job_id, on_stage, &app.state::<ExtensionState>().shutdown)?;
    let manifest = operation
        .read(read(&app, &source, operation.cancellation()))
        .await?;
    let marketplace = PluginMarketplace {
        id: uuid::Uuid::new_v4().to_string(),
        name: manifest.name,
        display_name: manifest.display_name,
        source,
        releases: manifest.releases,
    };
    operation.commit()?;
    sqlx::query("INSERT INTO marketplaces(id,document) VALUES($1,$2)")
        .bind(&marketplace.id)
        .bind(sqlx::types::Json(&marketplace))
        .execute(&database::pool(&app).await?)
        .await
        .map_err(|error| error.to_string())?;
    Ok(marketplace)
}

#[tauri::command]
pub async fn refresh_plugin_marketplace(
    app: AppHandle,
    marketplace_id: String,
    job_id: String,
    on_stage: Channel<PluginOperationStage>,
) -> AppResult<PluginMarketplace> {
    let operation =
        PluginOperation::begin(&job_id, on_stage, &app.state::<ExtensionState>().shutdown)?;
    let document: String = sqlx::query_scalar("SELECT document FROM marketplaces WHERE id=$1")
        .bind(&marketplace_id)
        .fetch_one(&database::pool(&app).await?)
        .await
        .map_err(|error| error.to_string())?;
    let mut marketplace: PluginMarketplace = database::decode(&document)?;
    let manifest = operation
        .read(read(&app, &marketplace.source, operation.cancellation()))
        .await?;
    if manifest.name != marketplace.name {
        return Err("marketplace-identity-changed".into());
    }
    marketplace.display_name = manifest.display_name;
    marketplace.releases = manifest.releases;
    let state = app.state::<ExtensionState>();
    let _guard = state.content_lock.lock().await;
    operation.commit()?;
    let updated = sqlx::query("UPDATE marketplaces SET document=$1 WHERE id=$2")
        .bind(sqlx::types::Json(&marketplace))
        .bind(marketplace_id)
        .execute(&database::pool(&app).await?)
        .await
        .map_err(|error| error.to_string())?;
    if updated.rows_affected() != 1 {
        return Err("marketplace-removed-during-refresh".into());
    }
    Ok(marketplace)
}

#[tauri::command]
pub async fn remove_plugin_marketplace(app: AppHandle, marketplace_id: String) -> AppResult<()> {
    if marketplace_id == super::bundled::MARKETPLACE_ID {
        return Err(AppError::invalid(
            "bundled-marketplace-is-application-managed",
        ));
    }
    let state = app.state::<ExtensionState>();
    let _guard = state.content_lock.lock().await;
    sqlx::query("DELETE FROM marketplaces WHERE id=$1")
        .bind(marketplace_id)
        .execute(&database::pool(&app).await?)
        .await
        .map_err(|error| error.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn search_plugins(app: AppHandle, query: String) -> AppResult<Vec<PluginMarketplace>> {
    let documents: Vec<String> =
        sqlx::query_scalar("SELECT document FROM marketplaces ORDER BY id")
            .fetch_all(&database::pool(&app).await?)
            .await
            .map_err(|error| error.to_string())?;
    let query = query.to_lowercase();
    let mut marketplaces = vec![super::bundled::marketplace()];
    marketplaces.extend(
        documents
            .iter()
            .map(|document| database::decode(document))
            .collect::<Result<Vec<PluginMarketplace>, _>>()?,
    );
    marketplaces
        .iter()
        .map(|marketplace| {
            let mut marketplace = marketplace.clone();
            marketplace.releases.retain(|release| {
                format!(
                    "{} {} {}",
                    release.publisher_id, release.name, release.description
                )
                .to_lowercase()
                .contains(&query)
            });
            Ok(marketplace)
        })
        .collect::<Result<Vec<_>, String>>()
        .map_err(AppError::from)
}

pub(super) async fn require_publication(
    app: &AppHandle,
    publication: &PluginPublication,
    source: &PluginSource,
) -> Result<(), String> {
    if publication.marketplace_id == super::bundled::MARKETPLACE_ID {
        return super::bundled::require_publication(publication, source);
    }
    let pool = database::pool(app).await?;
    let document: String = sqlx::query_scalar("SELECT document FROM marketplaces WHERE id=$1")
        .bind(&publication.marketplace_id)
        .fetch_one(&pool)
        .await
        .map_err(|error| error.to_string())?;
    let marketplace: PluginMarketplace = database::decode(&document)?;
    let release = marketplace
        .releases
        .iter()
        .find(|release| {
            release.publisher_id == publication.publisher_id
                && release.name == publication.plugin_name
        })
        .ok_or("marketplace-release-unavailable")?;
    if database::encode(&release.source)? != database::encode(source)? {
        return Err("marketplace-release-source-changed-review-again".into());
    }
    Ok(())
}

pub(super) async fn update_source(
    app: &AppHandle,
    plugin: &super::super::contracts::PluginInstallation,
) -> Result<PluginSource, String> {
    let Some(publication) = &plugin.publication else {
        return Ok(plugin.source.clone());
    };
    let document: String = sqlx::query_scalar("SELECT document FROM marketplaces WHERE id=$1")
        .bind(&publication.marketplace_id)
        .fetch_one(&database::pool(app).await?)
        .await
        .map_err(|error| error.to_string())?;
    let marketplace: PluginMarketplace = database::decode(&document)?;
    marketplace
        .releases
        .into_iter()
        .find(|release| {
            release.publisher_id == publication.publisher_id
                && release.name == publication.plugin_name
        })
        .map(|release| release.source)
        .ok_or_else(|| "marketplace-publication-unavailable-refresh-source".into())
}
