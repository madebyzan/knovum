use std::path::PathBuf;

use serde_json::Value;
use tauri::AppHandle;

use crate::foundation::app_error::AppError;
use crate::spaces::{assets::authorize_spaces_asset, previews::authorize_generated_preview};

use super::node::required_string;

/// The picture a node shows, which is what every reader of it reads: an image with its edits
/// rendered, a video by its cover. `revision::visual_revision` identifies the same input.
pub(super) async fn shown(app: &AppHandle, payload: &Value) -> Result<PathBuf, AppError> {
    match payload.get("kind").and_then(Value::as_str) {
        Some("image") => Ok(crate::spaces::assets::image::rendered_preview(app, payload).await?),
        Some("video") => {
            let path = std::path::Path::new(required_string(payload, "previewPath")?);
            Ok(if web_media_preview(payload) {
                authorize_spaces_asset(app, path)?
            } else {
                authorize_generated_preview(app, path)?
            })
        }
        _ => Err(AppError::invalid(
            "Only image and video nodes show a picture",
        )),
    }
}

fn web_media_preview(data: &Value) -> bool {
    data.get("origin")
        .and_then(|origin| origin.get("kind"))
        .and_then(Value::as_str)
        == Some("web")
}

#[cfg(test)]
#[path = "tests/picture.rs"]
mod tests;
