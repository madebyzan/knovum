use super::super::{
    content::{self, PackageFiles},
    contracts::{
        MarketplaceSource, PluginManifest, PluginMarketplace, PluginPublication, PluginRelease,
        PluginSource,
    },
    database, ExtensionState,
};
use crate::foundation::app_error::AppResult;
use tauri::{AppHandle, Manager};
use tokio_util::sync::CancellationToken;

pub(super) const MARKETPLACE_ID: &str = "knovum-bundled";

pub(super) struct BundledPlugin {
    pub id: &'static str,
    pub name: &'static str,
    pub manifest: &'static str,
}

const PACKAGES: &[BundledPlugin] = &[
    BundledPlugin {
        id: "421620e0-914f-4e20-a19b-51b9d606b2d6",
        name: "knowledge-work",
        manifest: include_str!("../../../resources/agent-plugins/knowledge-work/plugin.json"),
    },
    BundledPlugin {
        id: "b39bd049-006d-4ec6-a16c-548f8086e837",
        name: "evidence-research",
        manifest: include_str!("../../../resources/agent-plugins/evidence-research/plugin.json"),
    },
    BundledPlugin {
        id: "f040995e-5161-442f-8baa-c85b076147c4",
        name: "meeting-notes",
        manifest: include_str!("../../../resources/agent-plugins/meeting-notes/plugin.json"),
    },
    BundledPlugin {
        id: "a9c64ee2-ae4c-47d8-9b3f-b161918d6c71",
        name: "document-delivery",
        manifest: include_str!("../../../resources/agent-plugins/document-delivery/plugin.json"),
    },
    BundledPlugin {
        id: "c84da893-b0c2-401d-ae9b-6eef4f2cf0c7",
        name: "study-companion",
        manifest: include_str!("../../../resources/agent-plugins/study-companion/plugin.json"),
    },
    BundledPlugin {
        id: "d736b66e-8d80-4c55-8cdd-c9c25f7bc144",
        name: "github-work",
        manifest: include_str!("../../../resources/agent-plugins/github-work/plugin.json"),
    },
    BundledPlugin {
        id: "b6a217a5-00b8-488c-963a-3f2a6deff682",
        name: "notion-work",
        manifest: include_str!("../../../resources/agent-plugins/notion-work/plugin.json"),
    },
    BundledPlugin {
        id: "49cc93d1-79b6-459d-a4eb-7d0d6ddcb0c5",
        name: "linear-work",
        manifest: include_str!("../../../resources/agent-plugins/linear-work/plugin.json"),
    },
];

pub(super) fn require(name: &str) -> Result<&'static BundledPlugin, String> {
    PACKAGES
        .iter()
        .find(|package| package.name == name)
        .ok_or_else(|| "bundled-plugin-not-found".into())
}

pub(super) fn marketplace() -> PluginMarketplace {
    PluginMarketplace {
        id: MARKETPLACE_ID.into(),
        name: "knovum".into(),
        display_name: "Knovum".into(),
        source: MarketplaceSource::Bundled,
        releases: PACKAGES
            .iter()
            .map(|package| PluginRelease {
                publisher_id: "knovum".into(),
                name: package.name.into(),
                description: serde_json::from_str::<PluginManifest>(package.manifest)
                    .expect("bundled manifest is repository-owned and validated")
                    .description
                    .unwrap_or_default(),
                source: PluginSource::Bundled {
                    name: package.name.into(),
                },
            })
            .collect(),
    }
}

pub(super) fn require_publication(
    publication: &PluginPublication,
    source: &PluginSource,
) -> Result<(), String> {
    let package = require(&publication.plugin_name)?;
    if publication.publisher_id != "knovum"
        || !matches!(source, PluginSource::Bundled { name } if name == package.name)
    {
        return Err("bundled-publication-source-mismatch".into());
    }
    Ok(())
}

pub(super) async fn read(
    app: &AppHandle,
    name: &str,
    cancellation: &CancellationToken,
) -> AppResult<(PackageFiles, String)> {
    let package = require(name)?;
    #[cfg(debug_assertions)]
    let directory = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("resources/agent-plugins")
        .join(package.name);
    #[cfg(not(debug_assertions))]
    let directory = app
        .path()
        .resolve(
            format!("resources/agent-plugins/{}", package.name),
            tauri::path::BaseDirectory::Resource,
        )
        .map_err(|error| error.to_string())?;
    #[cfg(debug_assertions)]
    let _ = app;
    let cancellation = cancellation.clone();
    let files = crate::foundation::blocking::run(move || {
        super::sources::directory_files(&directory, &cancellation)
    })
    .await?;
    let revision = content::files_digest(&files);
    Ok((files, revision))
}

pub(crate) async fn reconcile_bundled_plugins(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<ExtensionState>();
    let _guard = state.content_lock.lock().await;
    for plugin in super::list(&database::pool(app).await?)
        .await?
        .into_iter()
        .filter(|plugin| plugin.builtin)
    {
        let package = require(&plugin.revision.manifest.name)?;
        if plugin.id != package.id {
            return Err("bundled-plugin-identity-mismatch".into());
        }
        let (files, revision) = read(app, package.name, &state.shutdown)
            .await
            .map_err(|error| error.message)?;
        if plugin.revision.digest == revision {
            continue;
        }
        let reviewed = super::packages::inspect(&files, revision)?;
        let change = super::installation::commit(
            app,
            package.id,
            PluginSource::Bundled {
                name: package.name.into(),
            },
            reviewed,
            files,
            true,
            plugin.publication,
        )
        .await?;
        if !change.cleanup_errors.is_empty() {
            return Err(change.cleanup_errors.join("\n"));
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests/bundled.rs"]
mod tests;
