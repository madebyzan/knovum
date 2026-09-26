use crate::command_error::CommandError;
use serde::Serialize;
use tauri::{AppHandle, Url};
use tauri_plugin_updater::UpdaterExt;

#[derive(Debug, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct UpdaterStatus {
    configured: bool,
}

#[derive(Debug, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMetadata {
    version: String,
    current_version: String,
}

fn release_config() -> Option<(&'static str, &'static str)> {
    let endpoint = option_env!("KNOVUM_UPDATER_ENDPOINT")?.trim();
    let pubkey = option_env!("KNOVUM_UPDATER_PUBKEY")?.trim();
    (!endpoint.is_empty() && !pubkey.is_empty()).then_some((endpoint, pubkey))
}

#[tauri::command]
pub fn updater_get_status() -> UpdaterStatus {
    UpdaterStatus {
        configured: release_config().is_some(),
    }
}

async fn resolve_update(app: &AppHandle) -> Result<Option<tauri_plugin_updater::Update>, String> {
    let Some((endpoint, pubkey)) = release_config() else {
        return Err("Updater is not configured for this build.".into());
    };
    app.updater_builder()
        .endpoints(vec![
            Url::parse(endpoint).map_err(|error| error.to_string())?
        ])
        .map_err(|error| error.to_string())?
        .pubkey(pubkey)
        .build()
        .map_err(|error| error.to_string())?
        .check()
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn updater_check(app: AppHandle) -> Result<Option<UpdateMetadata>, CommandError> {
    let update = resolve_update(&app).await?;
    Ok(update.map(|update| UpdateMetadata {
        version: update.version,
        current_version: update.current_version,
    }))
}

#[tauri::command]
pub async fn updater_install(app: AppHandle) -> Result<(), CommandError> {
    let update = resolve_update(&app)
        .await?
        .ok_or_else(|| "No update is currently available.".to_owned())?;

    update
        .download_and_install(|_, _| {}, || {})
        .await
        .map_err(|error| error.to_string())?;
    app.restart();
}
