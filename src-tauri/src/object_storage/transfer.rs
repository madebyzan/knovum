use chrono::Utc;
use sha2::{Digest, Sha256};

use super::settings::{Credentials, ObjectStorageSettings};
use super::{app, settings, sigv4};
use crate::foundation::app_error::{AppError, AppResult};

const MAX_UPLOAD_BYTES: usize = 64 * 1024 * 1024;

/// Uploads media to the user-configured object storage and returns the URL
/// models should be given. Fails with `not_configured` when the user has not
/// set object storage up.
pub(crate) async fn upload_media(bytes: Vec<u8>, media_type: &str) -> AppResult<String> {
    if bytes.is_empty() || bytes.len() > MAX_UPLOAD_BYTES {
        return Err(AppError::invalid(
            "Object storage uploads must be between 1 and 64 MiB",
        ));
    }
    let configuration = settings::load(app()?)?;
    let credentials = settings::read_credentials().await?;
    let key = object_key(&bytes, media_type);
    transfer("PUT", &configuration, &credentials, &key, Some(media_type), &bytes).await?;
    Ok(configuration.public_url(&key))
}

#[tauri::command]
pub(crate) async fn test_object_storage_connection() -> AppResult<()> {
    let configuration = settings::load(app()?)?;
    let credentials = settings::read_credentials().await?;
    let key = object_key(b"knovum-connection-test", "text/plain");
    transfer(
        "PUT",
        &configuration,
        &credentials,
        &key,
        Some("text/plain"),
        b"knovum-connection-test",
    )
    .await?;
    transfer("HEAD", &configuration, &credentials, &key, None, b"").await?;
    transfer("DELETE", &configuration, &credentials, &key, None, b"").await
}

async fn transfer(
    method: &str,
    configuration: &ObjectStorageSettings,
    credentials: &Credentials,
    key: &str,
    content_type: Option<&str>,
    payload: &[u8],
) -> AppResult<()> {
    let url = configuration
        .object_url(key)
        .parse::<reqwest::Url>()
        .map_err(|error| AppError::invalid(format!("Object storage URL is invalid: {error}")))?;
    sigv4::ensure_signable_url(&url)?;
    let signed = sigv4::authorization(
        method,
        &url,
        &configuration.region,
        credentials,
        content_type,
        payload,
        Utc::now(),
    )?;

    let mut request = super::client()
        .request(
            method
                .parse::<reqwest::Method>()
                .map_err(|error| AppError::internal(error.to_string()))?,
            url,
        )
        .header("x-amz-date", signed.amz_date)
        .header("x-amz-content-sha256", signed.payload_hash)
        .header(reqwest::header::AUTHORIZATION, signed.authorization);
    if let Some(content_type) = signed.content_type {
        request = request.header(reqwest::header::CONTENT_TYPE, content_type);
    }
    let response = request
        .body(payload.to_vec())
        .send()
        .await
        .map_err(|error| {
            AppError::new("object_storage_request_failed", error.without_url().to_string())
        })?;
    if !response.status().is_success() {
        let status = response.status();
        let detail = response.text().await.unwrap_or_default();
        return Err(AppError::new(
            "object_storage_request_failed",
            format!("Object storage {method} failed with {status}: {detail}"),
        ));
    }
    Ok(())
}

/// Content-addressed keys: uploading the same bytes twice stores one object,
/// and the digest keeps keys unique across months of the same content.
fn object_key(bytes: &[u8], media_type: &str) -> String {
    let digest = hex::encode(Sha256::digest(bytes));
    let now = Utc::now();
    format!(
        "knovum/{}/{}/{}.{}",
        now.format("%Y"),
        now.format("%m"),
        &digest[..32],
        extension(media_type)
    )
}

fn extension(media_type: &str) -> &'static str {
    match media_type {
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/webp" => "webp",
        "image/gif" => "gif",
        "video/mp4" => "mp4",
        "video/webm" => "webm",
        "audio/mpeg" => "mp3",
        "audio/wav" => "wav",
        "text/plain" => "txt",
        _ => "bin",
    }
}
