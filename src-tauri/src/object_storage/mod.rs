mod settings;
mod sigv4;
mod transfer;

use std::sync::OnceLock;

use tauri::{plugin::TauriPlugin, AppHandle, Wry};

use crate::foundation::app_error::{AppError, AppResult};

pub use settings::ObjectStorageSettings;

static APP: OnceLock<AppHandle> = OnceLock::new();
static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

pub(crate) fn install(app: &AppHandle) -> Result<(), String> {
    APP.set(app.clone())
        .map_err(|_| "object storage was initialized twice".to_owned())
}

fn app() -> AppResult<&'static AppHandle> {
    APP.get()
        .ok_or_else(|| AppError::internal("Object storage is not initialized"))
}

fn client() -> &'static reqwest::Client {
    CLIENT.get_or_init(|| {
        crate::foundation::http::client_builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .expect("object storage client builds")
    })
}

/// True when both the settings envelope and the credentials are present, so
/// media delivery can promise an uploaded URL.
pub async fn available() -> bool {
    let Ok(handle) = app() else {
        return false;
    };
    settings::load(handle).is_ok() && settings::read_credentials().await.is_ok()
}

/// Uploads media for a cloud model and returns the public URL to reference.
/// Errors with `not_configured` when object storage is unavailable, so callers
/// that cannot fall back to inline data can surface setup guidance.
pub async fn upload_media(bytes: Vec<u8>, media_type: &str) -> AppResult<String> {
    transfer::upload_media(bytes, media_type).await
}

pub fn plugin() -> TauriPlugin<Wry> {
    tauri::plugin::Builder::new("storage")
        .invoke_handler(tauri::generate_handler![
            settings::validate_object_storage_settings,
            transfer::test_object_storage_connection,
        ])
        .build()
}
