pub(super) mod commands;
mod connectors;
mod installation;
pub(in crate::extensions) mod marketplaces;
pub(in crate::extensions) mod operations;
mod packages;
pub(crate) mod process_configuration;
pub(crate) mod sources;
use super::{contracts::PluginSource, database, ExtensionState};
pub(crate) use installation::{list, load};
use tauri::{AppHandle, Manager};

const BUILTIN_ID: &str = "421620e0-914f-4e20-a19b-51b9d606b2d6";

pub(crate) async fn ensure_builtin(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<ExtensionState>();
    let _guard = state.content_lock.lock().await;
    let pool = database::pool(app).await?;
    #[cfg(debug_assertions)]
    let directory = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("resources/agent-plugins/knowledge-workflows");
    #[cfg(not(debug_assertions))]
    let directory = app
        .path()
        .resolve(
            "resources/agent-plugins/knowledge-workflows",
            tauri::path::BaseDirectory::Resource,
        )
        .map_err(|error| error.to_string())?;
    let files = crate::blocking::run(move || {
        sources::directory_files(&directory, &tokio_util::sync::CancellationToken::new())
    })
    .await?;
    let digest = super::content::files_digest(&files);
    let previous = sqlx::query_scalar::<_, String>("SELECT document FROM plugins WHERE id=$1")
        .bind(BUILTIN_ID)
        .fetch_optional(&pool)
        .await
        .map_err(|error| error.to_string())?;
    if previous
        .as_ref()
        .map(|document| database::decode::<super::contracts::PluginInstallation>(document))
        .transpose()?
        .is_some_and(|plugin| plugin.revision.digest == digest)
    {
        return Ok(());
    }
    let source = PluginSource::Directory {
        path: "builtin:knowledge-workflows".into(),
    };
    let reviewed = packages::inspect(&files, digest)?;
    installation::commit(app, BUILTIN_ID, source, reviewed, files, true, None).await?;
    Ok(())
}
