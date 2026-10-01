use std::time::Duration;

use tauri::AppHandle;
use tauri_plugin_updater::{Config, Update, UpdaterExt};

use crate::app_error::{AppError, AppResult};

pub(super) fn initialize(app: &AppHandle) -> AppResult<bool> {
    let Some(value) = app.config().plugins.0.get("updater") else {
        return Ok(false);
    };
    let config: Config = serde_json::from_value(value.clone())
        .map_err(|_| AppError::not_configured("Invalid embedded updater configuration"))?;
    validate(&config)?;
    if cfg!(debug_assertions) {
        return Ok(false);
    }
    app.plugin(tauri_plugin_updater::Builder::new().build())
        .map_err(|error| AppError::internal(error.to_string()))?;
    Ok(true)
}

pub(super) fn validate(config: &Config) -> AppResult<()> {
    if config.pubkey.trim().is_empty() || config.endpoints.len() != 1 {
        return Err(AppError::not_configured(
            "Updater requires a key and one stable endpoint",
        ));
    }
    if config.dangerous_insecure_transport_protocol
        || config.dangerous_accept_invalid_certs
        || config.dangerous_accept_invalid_hostnames
    {
        return Err(AppError::not_configured("Updater requires verified HTTPS"));
    }
    validate_url(&config.endpoints[0])
}

pub(super) fn validate_url(url: &tauri::Url) -> AppResult<()> {
    if url.scheme() != "https"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(AppError::invalid(
            "Updates require public, non-expiring HTTPS URLs",
        ));
    }
    Ok(())
}

pub(super) async fn check(app: &AppHandle) -> tauri_plugin_updater::Result<Option<Update>> {
    app.updater_builder()
        .timeout(Duration::from_secs(20))
        .configure_client(|client| {
            client
                .https_only(true)
                .connect_timeout(Duration::from_secs(10))
                .read_timeout(Duration::from_secs(60))
        })
        .build()?
        .check()
        .await
}
