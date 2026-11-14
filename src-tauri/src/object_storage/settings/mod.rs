use serde::{Deserialize, Serialize};

use crate::foundation::app_error::{AppError, AppResult};

const SETTINGS_FILE: &str = "settings.json";
const SETTINGS_KEY: &str = "storage.objectStorage";
const SCHEMA_VERSION: u8 = 1;

pub(crate) const ACCESS_KEY_ID_SECRET: &str = "integrations.storage.object-storage.access-key-id";
pub(crate) const SECRET_ACCESS_KEY_SECRET: &str =
    "integrations.storage.object-storage.secret-access-key";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Envelope<T> {
    schema_version: u8,
    value: T,
}

/// User-configured S3-compatible object storage. Credentials never live here;
/// they are stored in the secrets database under the fixed keys below.
#[derive(Clone, Debug, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename = "ObjectStorageSettings")]
pub struct ObjectStorageSettings {
    pub endpoint: String,
    pub region: String,
    pub bucket: String,
    pub public_base_url: Option<String>,
}

impl ObjectStorageSettings {
    /// Path-style object location used for signing and transfers.
    pub(crate) fn object_url(&self, key: &str) -> String {
        format!(
            "{}/{}/{}",
            self.endpoint.trim_end_matches('/'),
            self.bucket,
            key
        )
    }

    /// Location handed to models: the configured public base (a bucket domain
    /// or CDN) when present, otherwise the path-style transfer URL.
    pub(crate) fn public_url(&self, key: &str) -> String {
        match &self.public_base_url {
            Some(base) => format!("{}/{key}", base.trim_end_matches('/')),
            None => self.object_url(key),
        }
    }
}

pub(crate) struct Credentials {
    pub access_key_id: String,
    pub secret_access_key: String,
}

#[tauri::command]
pub(crate) fn validate_object_storage_settings(settings: ObjectStorageSettings) -> AppResult<()> {
    validate(&settings)
}

pub(crate) fn load(app: &tauri::AppHandle) -> AppResult<ObjectStorageSettings> {
    use tauri_plugin_store::StoreExt;
    let store = app
        .store(SETTINGS_FILE)
        .map_err(|error| AppError::internal(error.to_string()))?;
    let raw = store
        .get(SETTINGS_KEY)
        .ok_or_else(|| AppError::not_configured("Object storage is not configured"))?;
    let envelope: Envelope<ObjectStorageSettings> =
        serde_json::from_value(raw).map_err(|error| {
            AppError::invalid(format!("Object storage settings are invalid: {error}"))
        })?;
    if envelope.schema_version != SCHEMA_VERSION {
        return Err(AppError::invalid(
            "Object storage settings schema is unsupported",
        ));
    }
    validate(&envelope.value)?;
    Ok(envelope.value)
}

pub(crate) async fn read_credentials() -> AppResult<Credentials> {
    let access_key_id = read_secret(ACCESS_KEY_ID_SECRET).await?;
    let secret_access_key = read_secret(SECRET_ACCESS_KEY_SECRET).await?;
    Ok(Credentials {
        access_key_id,
        secret_access_key,
    })
}

async fn read_secret(key: &str) -> AppResult<String> {
    let value = crate::secrets::read_secret(key)
        .await
        .map_err(AppError::internal)?
        .unwrap_or_default();
    if value.trim().is_empty() {
        return Err(AppError::not_configured(
            "Object storage credentials are not configured",
        ));
    }
    Ok(value.trim().to_owned())
}

fn validate(settings: &ObjectStorageSettings) -> AppResult<()> {
    validate_https_url(&settings.endpoint, "endpoint")?;
    if settings.region.trim().is_empty() {
        return Err(AppError::invalid("Object storage region is required"));
    }
    validate_bucket(&settings.bucket)?;
    if let Some(base) = &settings.public_base_url {
        validate_https_url(base, "public base URL")?;
    }
    Ok(())
}

fn validate_https_url(value: &str, label: &str) -> AppResult<()> {
    let url = reqwest::Url::parse(value)
        .map_err(|_| AppError::invalid(format!("Object storage {label} is not a valid URL")))?;
    if url.scheme() != "https"
        || url.host_str().unwrap_or_default().is_empty()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err(AppError::invalid(format!(
            "Object storage {label} must be an https:// URL with a host"
        )));
    }
    Ok(())
}

fn validate_bucket(bucket: &str) -> AppResult<()> {
    let valid = (3..=63).contains(&bucket.len())
        && bucket.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b".-".contains(&byte)
        })
        && !bucket.starts_with(['-', '.'])
        && !bucket.ends_with(['-', '.'])
        && !bucket.contains("..");
    if valid {
        Ok(())
    } else {
        Err(AppError::invalid(
            "Object storage bucket must be 3-63 characters of lowercase letters, numbers, '.' or '-', without leading, trailing or repeated punctuation",
        ))
    }
}

#[cfg(test)]
mod tests;
