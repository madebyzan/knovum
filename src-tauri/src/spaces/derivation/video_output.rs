use std::{fs, path::PathBuf};

use serde_json::{Map, Value};
use tokio_util::sync::CancellationToken;

use crate::spaces::previews::GeneratedPreview;

pub struct RenderedVideo {
    pub path: String,
    pub size: i64,
    pub revision: String,
    pub cover: Option<GeneratedPreview>,
}

/// Reads what a node needs to know about the video that was just rendered for it, including the
/// cover its canvas shows. A rendered file reports no geometry of its own, so its cover carries
/// the aspect ratio as well — the same arrangement an imported video has.
pub fn inspect(
    app: &tauri::AppHandle,
    path: PathBuf,
    cancellation: &CancellationToken,
) -> Result<RenderedVideo, String> {
    let metadata = fs::metadata(&path).map_err(|error| error.to_string())?;
    if metadata.len() == 0 {
        return Err("FFmpeg created an empty video".into());
    }
    let revision = crate::file_inspection::content_revision(&path)?;
    let cover = crate::spaces::previews::generate_for_published_asset(app, &path, cancellation)?;
    Ok(RenderedVideo {
        path: path.to_string_lossy().into_owned(),
        size: metadata.len() as i64,
        revision,
        cover,
    })
}

/// Settles a derived video node onto the asset it now owns. Knowledge an Action preserves is the
/// Action's own business; this is only what the published file itself establishes.
pub fn apply_published(object: &mut Map<String, Value>, output: &RenderedVideo, now: i64) {
    object.insert("assetPath".into(), output.path.clone().into());
    object.insert("sourceRevision".into(), output.revision.clone().into());
    object.insert("size".into(), output.size.into());
    object.insert("modifiedAt".into(), now.into());
    object.remove("processingState");
    apply_cover(object, output.cover.as_ref());
}

pub fn apply_cover(object: &mut Map<String, Value>, cover: Option<&GeneratedPreview>) {
    match cover {
        Some(cover) => {
            object.insert("previewPath".into(), cover.path.clone().into());
            object.insert("intrinsicWidth".into(), cover.width.into());
            object.insert("intrinsicHeight".into(), cover.height.into());
        }
        None => {
            object.remove("previewPath");
        }
    }
}

pub const fn platform_h264_encoder() -> Result<&'static str, &'static str> {
    #[cfg(target_os = "macos")]
    return Ok("h264_videotoolbox");
    #[cfg(target_os = "windows")]
    return Ok("h264_mf");
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    Err("Video rendering is supported only on macOS and Windows")
}
