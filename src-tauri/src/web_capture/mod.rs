mod body;
mod media_type;

use reqwest::{
    header::{ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE, USER_AGENT},
    Client, Url,
};
use std::time::Duration;

pub use media_type::WebDocumentKind;

const MAX_DOCUMENT_BYTES: usize = 2 * 1024 * 1024;
const MAX_IMAGE_BYTES: usize = 8 * 1024 * 1024;
const USER_AGENT_VALUE: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136 Safari/537.36 Knovum/1.0";

pub(crate) enum CapturedWebResource {
    Document {
        final_url: Url,
        kind: WebDocumentKind,
        content: String,
    },
    Image {
        final_url: Url,
        media_type: mime::Mime,
        bytes: Vec<u8>,
    },
}

pub(crate) async fn capture(raw_url: &str) -> Result<CapturedWebResource, String> {
    let url = parse_url(raw_url)?;
    let client = Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|error| error.to_string())?;
    let mut response = client
        .get(url)
        .header(USER_AGENT, USER_AGENT_VALUE)
        .header(ACCEPT, "text/html,application/xhtml+xml,application/json,text/plain,image/avif,image/webp,image/*;q=0.8,*/*;q=0.5")
        .header(ACCEPT_LANGUAGE, "zh-CN,zh;q=0.9,en;q=0.8")
        .header(ACCEPT_ENCODING, "identity")
        .send()
        .await
        .map_err(|error| error.to_string())?
        .error_for_status()
        .map_err(|error| error.to_string())?;
    let final_url = response.url().clone();
    let response_type = media_type::response_media_type(&response);
    let (mut bytes, overflow) = body::read_body_prefix(&mut response, MAX_DOCUMENT_BYTES).await?;
    if let Some(media_type) =
        media_type::detect_image_media_type(response_type.clone(), &final_url, &bytes)
    {
        bytes.extend_from_slice(&overflow);
        body::read_body_remainder(&mut response, &mut bytes, MAX_IMAGE_BYTES).await?;
        return Ok(CapturedWebResource::Image {
            final_url,
            media_type,
            bytes,
        });
    }
    let kind = media_type::classify_document(response_type.as_ref(), &final_url, &bytes)
        .ok_or_else(|| "Web resource is neither a supported document nor an image".to_owned())?;
    Ok(CapturedWebResource::Document {
        final_url,
        kind,
        content: String::from_utf8_lossy(&bytes).into_owned(),
    })
}

pub(crate) async fn capture_image(raw_url: &str) -> Result<(Url, mime::Mime, Vec<u8>), String> {
    match capture(raw_url).await? {
        CapturedWebResource::Image {
            final_url,
            media_type,
            bytes,
        } => Ok((final_url, media_type, bytes)),
        CapturedWebResource::Document { .. } => Err("Web resource is not an image".into()),
    }
}

pub(crate) async fn capture_image_bounded(
    raw_url: &str,
    max_bytes: usize,
    allowed_host_suffix: &'static str,
) -> Result<(Url, mime::Mime, Vec<u8>), String> {
    if max_bytes == 0 {
        return Err("Web image size limit must be positive".into());
    }
    let url = parse_url(raw_url)?;
    validate_asset_url(&url)?;
    validate_host_suffix(&url, allowed_host_suffix)?;
    let client = Client::builder()
        .timeout(Duration::from_secs(180))
        .redirect(reqwest::redirect::Policy::custom(move |attempt| {
            if attempt.previous().len() >= 10
                || validate_asset_url(attempt.url()).is_err()
                || validate_host_suffix(attempt.url(), allowed_host_suffix).is_err()
            {
                attempt.stop()
            } else {
                attempt.follow()
            }
        }))
        .build()
        .map_err(|error| error.to_string())?;
    let mut response = client
        .get(url)
        .header(ACCEPT_ENCODING, "identity")
        .send()
        .await
        .map_err(|error| error.to_string())?
        .error_for_status()
        .map_err(|error| error.to_string())?;
    let final_url = response.url().clone();
    validate_asset_url(&final_url)?;
    validate_host_suffix(&final_url, allowed_host_suffix)?;
    let response_type = media_type::response_media_type(&response);
    let (mut bytes, overflow) =
        body::read_body_prefix(&mut response, max_bytes.min(MAX_DOCUMENT_BYTES)).await?;
    bytes.extend_from_slice(&overflow);
    body::read_body_remainder(&mut response, &mut bytes, max_bytes).await?;
    let media_type = media_type::detect_image_media_type(response_type, &final_url, &bytes)
        .ok_or("Web resource is not an image")?;
    Ok((final_url, media_type, bytes))
}

fn validate_host_suffix(url: &Url, suffix: &str) -> Result<(), String> {
    let host = url.host_str().ok_or("Web image URL has no host")?;
    let suffix = suffix.trim_start_matches('.');
    if host == suffix || host.ends_with(&format!(".{suffix}")) {
        Ok(())
    } else {
        Err("Web image URL host is not approved".into())
    }
}

fn validate_asset_url(url: &Url) -> Result<(), String> {
    if url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err("Web image URL must be credential-free HTTPS".into());
    }
    Ok(())
}

fn parse_url(raw_url: &str) -> Result<Url, String> {
    let url = Url::parse(raw_url).map_err(|error| error.to_string())?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err("Web resource URL must use HTTP or HTTPS".into());
    }
    Ok(url)
}

#[cfg(test)]
mod tests;
