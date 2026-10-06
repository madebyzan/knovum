use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Runtime};
use tauri_plugin_store::{Store, StoreExt};

const FILE: &str = "local-resources.json";
const KEY: &str = "downloadRegion";
static ACCESS: Mutex<()> = Mutex::new(());

#[derive(Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq, Hash, Debug, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub(crate) enum ResourceDownloadRegion {
    #[default]
    Global,
    China,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Envelope {
    schema_version: u8,
    value: ResourceDownloadRegion,
}

pub(crate) fn region<R: Runtime>(app: &AppHandle<R>) -> Result<ResourceDownloadRegion, String> {
    let _access = ACCESS.lock().map_err(|error| error.to_string())?;
    load_store(app).map(|(_, region)| region)
}

fn load_store<R: Runtime>(
    app: &AppHandle<R>,
) -> Result<(Arc<Store<R>>, ResourceDownloadRegion), String> {
    let store = app
        .store_builder(FILE)
        .disable_auto_save()
        .build()
        .map_err(|error| error.to_string())?;
    // Store construction ignores load errors; reload explicitly checks disk state.
    match store.reload_ignore_defaults() {
        Ok(()) => {}
        Err(tauri_plugin_store::Error::Io(error))
            if error.kind() == std::io::ErrorKind::NotFound =>
        {
            store.clear();
        }
        Err(error) => return Err(error.to_string()),
    };
    let region = store.get(KEY).map(decode).transpose()?.unwrap_or_default();
    Ok((store, region))
}

fn decode(raw: serde_json::Value) -> Result<ResourceDownloadRegion, String> {
    let envelope: Envelope = serde_json::from_value(raw)
        .map_err(|error| format!("Download region settings are invalid: {error}"))?;
    if envelope.schema_version != 1 {
        return Err("Download region settings schema is unsupported".into());
    }
    Ok(envelope.value)
}

#[tauri::command]
pub(crate) fn set_download_region(
    app: AppHandle,
    region: ResourceDownloadRegion,
) -> Result<(), crate::app_error::AppError> {
    let _access = ACCESS
        .lock()
        .map_err(|error| crate::app_error::AppError::internal(error.to_string()))?;
    let (store, _) = load_store(&app).map_err(crate::app_error::AppError::internal)?;
    store.set(
        KEY,
        serde_json::to_value(Envelope {
            schema_version: 1,
            value: region,
        })
        .map_err(|error| crate::app_error::AppError::internal(error.to_string()))?,
    );
    store
        .save()
        .map_err(|error| crate::app_error::AppError::internal(error.to_string()))?;
    Ok(())
}

#[cfg(test)]
#[path = "tests/preferences.rs"]
mod tests;
