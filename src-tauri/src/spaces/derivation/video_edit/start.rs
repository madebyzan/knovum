use serde_json::Value;
use tauri::AppHandle;

use super::{fonts, validate_project, RenderVideoEditRequest, VideoEditProject};
use crate::spaces::{
    db,
    derivation::{inputs, pending_output},
};

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
    crate::spaces::knowledge::validation::validate_same_space_node(
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
    if data.get("videoEditProject")
        != Some(&serde_json::to_value(&request.project).map_err(db::json_error)?)
    {
        return Err("Video edit project changed before rendering".into());
    }
    let path = data
        .get("assetPath")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or("Video asset is unavailable")?;
    let output: String = sqlx::query_scalar(
        "SELECT data FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND kind='video' AND deleted_at IS NULL",
    )
    .bind(&request.output_node_id)
    .bind(&request.space_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db::sql_error)?
    .ok_or("Video edit output is unavailable")?;
    let output: Value = serde_json::from_str(&output).map_err(db::json_error)?;
    pending_output::validate_separate_asset(&output, path)?;
    let now = crate::foundation::clock::unix_time_millis()?;
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
            revision_basis: inputs::RevisionBasis::Rendered,
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
    if request.id.is_empty() || request.id.len() > 64 {
        return Err("Video edit request is invalid".into());
    }
    validate_project(&request.project, &request.source_revision)?;
    if let Some(item) = request
        .project
        .text_track
        .items
        .iter()
        .find(|item| !fonts::is_available(&item.style.family, item.style.weight))
    {
        return Err(format!(
            "Font {} at weight {} is not installed on this computer",
            item.style.family, item.style.weight
        ));
    }
    Ok(())
}
