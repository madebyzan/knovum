use crate::command_error::CommandError;
use serde::Serialize;
use std::fs;
use uuid::Uuid;

use crate::web_capture::{CapturedWebResource, WebDocumentKind};

use super::super::asset_directory;

#[derive(Serialize)]
#[serde(
    tag = "resourceKind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum WebpageResponse {
    Document {
        final_url: String,
        document_kind: WebDocumentKind,
        content: String,
    },
    Image {
        final_url: String,
        asset_path: String,
    },
}

#[tauri::command]
pub async fn capture_web_resource(
    app: tauri::AppHandle,
    url: String,
) -> Result<WebpageResponse, CommandError> {
    match crate::web_capture::capture(&url).await? {
        CapturedWebResource::Document {
            final_url,
            kind,
            content,
        } => Ok(WebpageResponse::Document {
            final_url: final_url.to_string(),
            document_kind: kind,
            content,
        }),
        CapturedWebResource::Image {
            final_url,
            media_type,
            bytes,
        } => Ok(WebpageResponse::Image {
            final_url: final_url.to_string(),
            asset_path: persist(&app, &bytes, &media_type)?,
        }),
    }
}

#[tauri::command]
pub async fn import_web_image(app: tauri::AppHandle, url: String) -> Result<String, CommandError> {
    let (_, media_type, bytes) = crate::web_capture::capture_derived_image(&url).await?;
    Ok((persist(&app, &bytes, &media_type))?)
}

fn persist(
    app: &tauri::AppHandle,
    bytes: &[u8],
    media_type: &mime::Mime,
) -> Result<String, String> {
    let extension = web_image_extension(media_type)?;
    let directory = asset_directory(app)?.join("web");
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let target = directory.join(format!("{}.{extension}", Uuid::new_v4()));
    fs::write(&target, bytes).map_err(|error| error.to_string())?;
    Ok(target.to_string_lossy().into_owned())
}

fn web_image_extension(media_type: &mime::Mime) -> Result<&'static str, String> {
    match media_type.essence_str() {
        "image/vnd.microsoft.icon" | "image/x-icon" => Ok("ico"),
        _ => mime_guess::get_mime_extensions(media_type)
            .and_then(|values| values.first())
            .copied()
            .ok_or_else(|| format!("Web image type {media_type} has no file extension")),
    }
}

#[cfg(test)]
mod tests;
