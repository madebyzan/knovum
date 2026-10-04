use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use tauri::{AppHandle, Runtime};

use super::super::contracts::SpeechSegmentRenderOutput;
use crate::{file_inspection::verify_content_revision, spaces::assets};

pub(super) fn verified_source<R: Runtime>(
    app: &AppHandle<R>,
    path: &str,
    revision: &str,
) -> Result<PathBuf, String> {
    let path = assets::authorize_spaces_asset(app, Path::new(path))?;
    verify_content_revision(&path, revision)?;
    Ok(path)
}

pub(super) fn assembly_digest(
    renders: &[SpeechSegmentRenderOutput],
    windows: &[(Option<i64>, Option<i64>)],
    config: &serde_json::Value,
) -> String {
    let mut digest = Sha256::new();
    for (render, window) in renders.iter().zip(windows) {
        digest.update(render.segment_id.as_bytes());
        digest.update(render.sha256.as_bytes());
        digest.update(window.0.unwrap_or(-1).to_le_bytes());
        digest.update(window.1.unwrap_or(-1).to_le_bytes());
    }
    if let Ok(encoded) = serde_json::to_vec(config) {
        digest.update(encoded);
    }
    format!("{:x}", digest.finalize())
}
