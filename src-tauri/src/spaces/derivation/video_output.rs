use std::{fs, path::PathBuf};

use serde_json::{Map, Value};
use tokio_util::sync::CancellationToken;

use crate::spaces::{assets::published_asset::PublishedAsset, previews::GeneratedPreview};

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

/// Settles a derived video node onto the asset it now owns.
pub fn apply_published(
    object: &mut Map<String, Value>,
    output: &RenderedVideo,
    now: i64,
) -> Result<(), String> {
    PublishedAsset {
        path: &output.path,
        revision: output.revision.clone(),
        size: Some(output.size as u64),
        cover: output.cover.as_ref(),
    }
    .settle_onto(object, now)
}

/// Discards a rendered video that will not be published, together with the cover made from it.
pub async fn discard(app: &tauri::AppHandle, path: &str) -> Result<(), String> {
    match tokio::fs::remove_file(path).await {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.to_string()),
    }
    crate::spaces::previews::remove_generated(app, std::path::Path::new(path))
}

pub const fn platform_h264_encoder() -> Result<&'static str, &'static str> {
    #[cfg(target_os = "macos")]
    return Ok("h264_videotoolbox");
    #[cfg(target_os = "windows")]
    return Ok("h264_mf");
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    Err("Video rendering is supported only on macOS and Windows")
}
