use std::sync::atomic::{AtomicU8, Ordering};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Runtime};
use tauri_plugin_store::StoreExt;

use crate::foundation::app_error::AppError;

pub(crate) const STORE_KEY: &str = "localResources.downloadSource";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub enum DownloadSource {
    #[default]
    #[serde(rename = "huggingface")]
    HuggingFace,
    #[serde(rename = "hf-mirror")]
    HfMirror,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Envelope {
    schema_version: u8,
    value: DownloadSource,
}

static ACTIVE_SOURCE: AtomicU8 = AtomicU8::new(0);

impl DownloadSource {
    fn code(self) -> u8 {
        match self {
            Self::HuggingFace => 0,
            Self::HfMirror => 1,
        }
    }

    fn from_code(code: u8) -> Self {
        match code {
            1 => Self::HfMirror,
            _ => Self::HuggingFace,
        }
    }
}

pub(crate) fn active() -> DownloadSource {
    DownloadSource::from_code(ACTIVE_SOURCE.load(Ordering::Relaxed))
}

pub(crate) fn set_active(source: DownloadSource) {
    ACTIVE_SOURCE.store(source.code(), Ordering::Relaxed);
}

pub(crate) fn validate_persisted_value(value: serde_json::Value) -> Result<(), String> {
    serde_json::from_value::<DownloadSource>(value)
        .map(drop)
        .map_err(|error| format!("Invalid local model download source: {error}"))
}

pub(crate) fn load<R: Runtime>(app: &AppHandle<R>) -> DownloadSource {
    let source = app
        .store("settings.json")
        .ok()
        .and_then(|store| store.get(STORE_KEY))
        .and_then(|raw| serde_json::from_value::<Envelope>(raw).ok())
        .filter(|envelope| envelope.schema_version == 1)
        .map(|envelope| envelope.value)
        .unwrap_or_default();
    set_active(source);
    source
}

#[tauri::command]
pub async fn get_download_source(app: AppHandle) -> Result<DownloadSource, AppError> {
    Ok(crate::foundation::blocking::run(move || Ok::<_, String>(load(&app))).await?)
}

#[tauri::command]
pub async fn set_download_source(app: AppHandle, source: DownloadSource) -> Result<(), AppError> {
    crate::foundation::blocking::run(move || {
        let store = app
            .store("settings.json")
            .map_err(|error| error.to_string())?;
        let payload = serde_json::to_value(Envelope {
            schema_version: 1,
            value: source,
        })
        .map_err(|error| error.to_string())?;
        store.set(STORE_KEY, payload);
        store.save().map_err(|error| error.to_string())?;
        set_active(source);
        Ok::<_, String>(())
    })
    .await?;
    Ok(())
}
