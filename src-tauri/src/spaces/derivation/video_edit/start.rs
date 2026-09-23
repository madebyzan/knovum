use serde_json::Value;
use tauri::AppHandle;
use uuid::Uuid;

use super::{RenderVideoEditRequest, VideoEditProject};
use crate::spaces::{db, derivation::inputs, knowledge::contracts};

pub struct VideoEditSource {
    pub path: String,
    pub project: VideoEditProject,
}

pub async fn start(
    app: &AppHandle,
    request: &RenderVideoEditRequest,
) -> Result<VideoEditSource, String> {
    validate_request(request)?;
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    contracts::validate_same_space_node(
        &mut tx,
        &request.space_id,
        &request.source_node_id,
        &request.output_node_id,
    )
    .await?;
    let encoded: String = sqlx::query_scalar("SELECT data FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND kind='video' AND deleted_at IS NULL")
        .bind(&request.source_node_id).bind(&request.space_id).fetch_optional(&mut *tx).await.map_err(db::sql_error)?
        .ok_or("Video source is unavailable")?;
    let data: Value = serde_json::from_str(&encoded).map_err(db::json_error)?;
    if data.get("sourceRevision").and_then(Value::as_str) != Some(&request.source_revision) {
        return Err("Video source changed before rendering".into());
    }
    let path = data
        .get("assetPath")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or("Video asset is unavailable")?;
    let now = crate::clock::unix_time_millis()?;
    sqlx::query("INSERT INTO spaces_knowledge_derivations(id,space_id,output_node_id,action_kind,action_version,engine,status,created_at,updated_at) VALUES($1,$2,$3,'render-video-edit',1,'ffmpeg:filter-complex','running',$4,$4)")
        .bind(&request.id).bind(&request.space_id).bind(&request.output_node_id).bind(now)
        .execute(&mut *tx).await.map_err(db::sql_error)?;
    inputs::insert(
        &mut tx,
        &request.id,
        &[inputs::DerivationInput {
            node_id: &request.source_node_id,
            role: "primary",
            source_revision: &request.source_revision,
        }],
    )
    .await?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(VideoEditSource {
        path: path.into(),
        project: request.project.clone(),
    })
}

fn validate_request(request: &RenderVideoEditRequest) -> Result<(), String> {
    if request.id.is_empty() || request.id.len() > 64 || request.project.version != 1 {
        return Err("Video edit request is invalid".into());
    }
    if request.project.source_revision != request.source_revision
        || request.project.video_track.clips.is_empty()
        || request.project.video_track.clips.len() > 500
        || request.project.text_track.items.len() > 500
    {
        return Err("Video edit project does not match the source".into());
    }
    if request.project.video_track.clips.iter().any(|clip| {
        Uuid::parse_str(&clip.id).is_err()
            || clip.end_ms <= clip.start_ms
            || !(0.0..=2.0).contains(&clip.volume)
    }) {
        return Err("Video edit project contains an invalid clip".into());
    }
    if request.project.text_track.items.iter().any(|item| {
        Uuid::parse_str(&item.id).is_err()
            || item.end_ms <= item.start_ms
            || item.text.trim().is_empty()
            || item.text.len() > 500
            || item.style.as_ref().is_some_and(|style| {
                !(16..=96).contains(&style.size)
                    || style.color.len() != 7
                    || !style.color.starts_with('#')
                    || !style.color[1..]
                        .bytes()
                        .all(|byte| byte.is_ascii_hexdigit())
            })
    }) {
        return Err("Video edit project contains invalid timed text".into());
    }
    if !matches!(request.project.transform.rotation, 0 | 90 | 180 | 270) {
        return Err("Video rotation is invalid".into());
    }
    Ok(())
}
