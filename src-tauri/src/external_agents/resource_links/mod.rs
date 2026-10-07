use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use agent_client_protocol::{schema::v1::ResourceLink, Error};
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;

use crate::public_network::public_address;

const MAX_REMOTE_RESOURCE_BYTES: usize = 64 * 1024 * 1024;
const MAX_REMOTE_IMAGE_BYTES: usize = 64 * 1024 * 1024;
const MAX_REMOTE_AUDIO_BYTES: usize = 128 * 1024 * 1024;
const MAX_REMOTE_VIDEO_BYTES: usize = 512 * 1024 * 1024;

pub(super) async fn materialize(
    workspace: &Path,
    link: ResourceLink,
    output_identity: Option<&str>,
) -> Result<Option<PathBuf>, Error> {
    acquire(workspace, link, output_identity)
        .await
        .map_err(|error| Error::invalid_request().data(error))
}

async fn acquire(
    workspace: &Path,
    link: ResourceLink,
    output_identity: Option<&str>,
) -> Result<Option<PathBuf>, String> {
    let url = tauri::Url::parse(&link.uri).map_err(|error| error.to_string())?;
    if url.scheme() == "file" {
        let path = url.to_file_path().map_err(|()| "Invalid Agent file URL")?;
        let metadata = tokio::fs::symlink_metadata(&path)
            .await
            .map_err(|error| error.to_string())?;
        let canonical = tokio::fs::canonicalize(&path)
            .await
            .map_err(|error| error.to_string())?;
        let root = tokio::fs::canonicalize(workspace)
            .await
            .map_err(|error| error.to_string())?;
        if !metadata.is_file() || metadata.file_type().is_symlink() || !canonical.starts_with(&root)
        {
            return Err("Agent output file must be a regular file inside its workspace".into());
        }
        return Ok(Some(canonical));
    }
    require_https(&url)?;
    let response = remote_response(url.clone()).await?;
    let response_content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok());
    let extension = remote_extension(
        response.url(),
        response_content_type,
        link.mime_type.as_deref(),
    )?;
    let maximum_bytes = remote_size_limit(response_content_type, link.mime_type.as_deref());
    let identity = Sha256::digest(url.as_str().as_bytes());
    let base = output_identity
        .map(str::to_owned)
        .unwrap_or_else(|| format!("resource-{identity}", identity = hex::encode(identity)));
    persist_response(workspace, response, &base, &extension, maximum_bytes)
        .await
        .map(Some)
}

async fn persist_response(
    workspace: &Path,
    mut response: reqwest::Response,
    base: &str,
    extension: &str,
    maximum_bytes: usize,
) -> Result<PathBuf, String> {
    if response
        .content_length()
        .is_some_and(|length| length > maximum_bytes as u64)
    {
        return Err("Agent remote output exceeded its size limit".into());
    }
    let pending = workspace.join(format!(".{base}-{}.partial", uuid::Uuid::new_v4()));
    let mut file = tokio::fs::File::create(&pending)
        .await
        .map_err(|error| error.to_string())?;
    let mut content_identity = Sha256::new();
    let mut written = 0_usize;
    let publication = async {
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|error| error.without_url().to_string())?
        {
            written = written.saturating_add(chunk.len());
            if written > maximum_bytes {
                return Err("Agent remote output exceeded its size limit".into());
            }
            content_identity.update(&chunk);
            file.write_all(&chunk)
                .await
                .map_err(|error| error.to_string())?;
        }
        file.sync_all().await.map_err(|error| error.to_string())?;
        drop(file);
        let content_identity = content_identity.finalize();
        let target = workspace.join(format!(
            "{base}-{content_identity}.{extension}",
            content_identity = hex::encode(content_identity)
        ));
        tokio::fs::rename(&pending, &target)
            .await
            .map_err(|error| error.to_string())?;
        Ok(target)
    }
    .await;
    if publication.is_err() {
        let _ = tokio::fs::remove_file(&pending).await;
    }
    publication
}

fn remote_size_limit(
    response_content_type: Option<&str>,
    declared_mime_type: Option<&str>,
) -> usize {
    let response_mime = response_content_type.and_then(|value| value.parse::<mime::Mime>().ok());
    response_mime
        .as_ref()
        .and_then(|mime| {
            if mime.essence_str() == "application/octet-stream" {
                declared_mime_type.and_then(mime_size_limit)
            } else {
                mime_size_limit(mime.essence_str())
            }
        })
        .or_else(|| declared_mime_type.and_then(mime_size_limit))
        .unwrap_or(MAX_REMOTE_RESOURCE_BYTES)
}

fn mime_size_limit(value: &str) -> Option<usize> {
    let mime = value.parse::<mime::Mime>().ok()?;
    match mime.type_().as_str() {
        "image" => Some(MAX_REMOTE_IMAGE_BYTES),
        "audio" => Some(MAX_REMOTE_AUDIO_BYTES),
        "video" => Some(MAX_REMOTE_VIDEO_BYTES),
        _ => Some(MAX_REMOTE_RESOURCE_BYTES),
    }
}

fn remote_extension(
    final_url: &tauri::Url,
    response_content_type: Option<&str>,
    declared_mime_type: Option<&str>,
) -> Result<String, String> {
    let response_extensions = response_content_type
        .and_then(|value| value.parse::<mime::Mime>().ok())
        .and_then(|value| mime_guess::get_mime_extensions_str(value.essence_str()));
    Path::new(final_url.path())
        .extension()
        .and_then(|value| value.to_str())
        .filter(|value| value.len() <= 10 && value.chars().all(|c| c.is_ascii_alphanumeric()))
        .filter(|value| response_extensions.is_none_or(|extensions| extensions.contains(value)))
        .map(str::to_owned)
        .or_else(|| {
            response_extensions.and_then(|values| values.first().map(|value| (*value).into()))
        })
        .or_else(|| {
            declared_mime_type
                .and_then(|value| value.parse::<mime::Mime>().ok())
                .and_then(|value| mime_guess::get_mime_extensions_str(value.essence_str()))
                .and_then(|values| values.first().map(|value| (*value).into()))
        })
        .ok_or_else(|| "Agent resource link has no file type".into())
}

async fn remote_response(mut url: tauri::Url) -> Result<reqwest::Response, String> {
    for _ in 0..=5 {
        require_https(&url)?;
        let host = url.host_str().ok_or("Agent resource has no host")?;
        let addresses: Vec<_> =
            tokio::net::lookup_host((host, url.port_or_known_default().unwrap_or(443)))
                .await
                .map_err(|error| error.to_string())?
                .collect();
        if addresses.is_empty()
            || addresses
                .iter()
                .any(|address| !public_address(address.ip()))
        {
            return Err("Agent remote output must resolve only to public addresses".into());
        }
        let client = reqwest::Client::builder()
            .no_proxy()
            .timeout(Duration::from_secs(120))
            .redirect(reqwest::redirect::Policy::none())
            .resolve_to_addrs(host, &addresses)
            .build()
            .map_err(|error| error.to_string())?;
        let response = client
            .get(url.clone())
            .send()
            .await
            .map_err(|error| error.without_url().to_string())?;
        if response.status().is_redirection() {
            let location = response
                .headers()
                .get(reqwest::header::LOCATION)
                .and_then(|value| value.to_str().ok())
                .ok_or("Agent output redirect has no location")?;
            url = url.join(location).map_err(|error| error.to_string())?;
        } else {
            return response
                .error_for_status()
                .map_err(|error| error.without_url().to_string());
        }
    }
    Err("Agent output exceeded redirect limit".into())
}

fn require_https(url: &tauri::Url) -> Result<(), String> {
    if url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err("Agent remote output must use credential-free HTTPS".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests;
