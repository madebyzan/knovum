use std::time::Duration;

use chrono::Utc;
use object_store::aws::AmazonS3Builder;
use object_store::path::Path;
use object_store::{
    Attribute, Attributes, ClientOptions, ObjectStore, ObjectStoreExt, PutOptions, PutPayload,
};
use sha2::{Digest, Sha256};

use super::settings::{Credentials, ObjectStorageSettings};
use super::{app, settings};
use crate::foundation::app_error::{AppError, AppResult};
use crate::foundation::http::USER_AGENT;

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
    build_store(&configuration, &credentials)?
        .put_opts(
            &Path::from(key.as_str()),
            PutPayload::from(bytes),
            PutOptions {
                attributes: Attributes::from_iter([(
                    Attribute::ContentType,
                    media_type.to_owned(),
                )]),
                ..PutOptions::default()
            },
        )
        .await
        .map_err(request_error)?;
    Ok(configuration.public_url(&key))
}

#[tauri::command]
pub(crate) async fn test_object_storage_connection() -> AppResult<()> {
    let configuration = settings::load(app()?)?;
    let credentials = settings::read_credentials().await?;
    let store = build_store(&configuration, &credentials)?;
    let path = Path::from(object_key(b"knovum-connection-test", "text/plain"));
    store
        .put(&path, PutPayload::from_static(b"knovum-connection-test"))
        .await
        .map_err(request_error)?;
    store.head(&path).await.map_err(request_error)?;
    store.delete(&path).await.map_err(request_error)
}

fn build_store(
    configuration: &ObjectStorageSettings,
    credentials: &Credentials,
) -> AppResult<impl ObjectStore> {
    AmazonS3Builder::new()
        .with_endpoint(&configuration.endpoint)
        .with_region(&configuration.region)
        .with_bucket_name(&configuration.bucket)
        .with_access_key_id(&credentials.access_key_id)
        .with_secret_access_key(&credentials.secret_access_key)
        .with_virtual_hosted_style_request(false)
        .with_client_options(
            ClientOptions::default()
                .with_user_agent(reqwest::header::HeaderValue::from_static(USER_AGENT))
                .with_timeout(Duration::from_secs(300)),
        )
        .build()
        .map_err(|error| {
            AppError::invalid(format!("Object storage configuration is invalid: {error}"))
        })
}

fn request_error(error: object_store::Error) -> AppError {
    AppError::new("object_storage_request_failed", error.to_string())
}

/// Content-addressed keys: within a month, uploading the same bytes twice
/// stores one object. The year/month prefix shards keys for S3 throughput, so
/// identical content uploaded in different months gets distinct keys.
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
