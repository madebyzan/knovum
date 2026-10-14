use serde_json::Value;
use sqlx::{Sqlite, Transaction};
use std::path::PathBuf;

use super::{contracts::VideoAssetKind, VideoEditProject};
use crate::spaces::{assets::authorize_spaces_asset, db};

pub(super) struct ResolvedAsset {
    pub node_id: String,
    pub path: String,
}

pub(super) async fn resolve(
    app: &tauri::AppHandle,
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    project: &VideoEditProject,
) -> Result<Vec<ResolvedAsset>, String> {
    let mut resolved = Vec::new();
    for asset in used_assets(project) {
        let data = load(tx, space_id, asset).await?;
        let path = data
            .get("assetPath")
            .and_then(Value::as_str)
            .ok_or("Video material has no managed asset")?;
        let path = authorize_spaces_asset(app, &PathBuf::from(path))?;
        if !path.is_file() {
            return Err("Video material must be a managed file".into());
        }
        resolved.push(ResolvedAsset {
            node_id: asset.node_id.clone(),
            path: path.to_string_lossy().into_owned(),
        });
    }
    Ok(resolved)
}
pub(super) fn used_assets(
    project: &VideoEditProject,
) -> impl Iterator<Item = &super::contracts::VideoAsset> {
    project
        .assets
        .iter()
        .enumerate()
        .filter(|(index, asset)| {
            *index == 0
                || project
                    .video_track
                    .clips
                    .iter()
                    .any(|clip| clip.asset_id == asset.node_id)
                || project
                    .audio_track
                    .clips
                    .iter()
                    .any(|clip| clip.asset_id == asset.node_id)
        })
        .map(|(_, asset)| asset)
}
async fn load(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    asset: &super::contracts::VideoAsset,
) -> Result<Value, String> {
    let row: Option<(String, String)> = sqlx::query_as(
        "SELECT kind,data FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND deleted_at IS NULL",
    )
    .bind(&asset.node_id)
    .bind(space_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let (kind, encoded) = row.ok_or("Video material is deleted or outside this Space")?;
    let expected = match asset.kind {
        VideoAssetKind::Video => "video",
        VideoAssetKind::Image => "image",
        VideoAssetKind::Audio => "audio",
    };
    let data: Value = serde_json::from_str(&encoded).map_err(db::json_error)?;
    if kind != expected
        || data.get("sourceRevision").and_then(Value::as_str)
            != Some(asset.source_revision.as_str())
        || data.get("processingState").is_some()
    {
        return Err("Video material changed or is not ready".into());
    }
    Ok(data)
}

#[cfg(test)]
mod tests;
