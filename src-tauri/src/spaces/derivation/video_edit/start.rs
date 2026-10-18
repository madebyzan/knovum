use serde_json::Value;
use tauri::AppHandle;

use super::{fonts, validate_project, RenderVideoEditRequest, VideoEditProject};
use crate::spaces::{
    db,
    derivation::{inputs, pending_output, relations},
};

pub(super) struct VideoEditSource {
    pub(super) assets: Vec<super::sources::ResolvedAsset>,
    pub(super) project: VideoEditProject,
}

pub(super) async fn start(
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
    validate_saved_project(&data, &request.project)?;
    if request.project.assets[0].node_id != request.source_node_id {
        return Err("Video project belongs to a different source node".into());
    }
    let assets = super::sources::resolve(app, &mut tx, &request.space_id, &request.project).await?;
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
    for asset in &assets {
        pending_output::validate_separate_asset(&output, &asset.path)?;
    }
    let now = crate::foundation::clock::unix_time_millis()?;
    sqlx::query("INSERT INTO spaces_knowledge_derivations(id,space_id,output_node_id,action_kind,action_version,engine,status,created_at,updated_at) VALUES($1,$2,$3,'render-video-edit',1,'ffmpeg:filter-complex','running',$4,$4)")
        .bind(&request.id).bind(&request.space_id).bind(&request.output_node_id).bind(now)
        .execute(&mut *tx).await.map_err(db::sql_error)?;
    let inputs = super::sources::used_assets(&request.project)
        .map(|asset| inputs::DerivationInput {
            node_id: &asset.node_id,
            role: if asset.node_id == request.source_node_id {
                "primary"
            } else {
                "source"
            },
            source_revision: &asset.source_revision,
            revision_basis: if asset.node_id == request.source_node_id {
                inputs::RevisionBasis::Rendered
            } else {
                inputs::RevisionBasis::Asset
            },
        })
        .collect::<Vec<_>>();
    inputs::insert(&mut tx, &request.id, &inputs).await?;
    // The output node was reserved before this render was asked for, so its lineage is recorded
    // here, when the work starts, rather than when it settles: a rendering node is already
    // connected to every asset it is made of.
    for input in &inputs {
        relations::derives_from(
            &mut tx,
            &request.space_id,
            &request.output_node_id,
            input.node_id,
            "Video rendered from a non-destructive edit project",
            now,
        )
        .await?;
    }
    tx.commit().await.map_err(db::sql_error)?;
    Ok(VideoEditSource {
        assets,
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

pub(super) fn validate_saved_project(
    data: &Value,
    requested: &VideoEditProject,
) -> Result<(), String> {
    let saved = data
        .get("videoEditProject")
        .ok_or("Saved video edit project is unavailable")?;
    let saved: VideoEditProject = serde_json::from_value(saved.clone())
        .map_err(|error| format!("Saved video edit project is invalid: {error}"))?;
    if &saved != requested {
        return Err("Video edit project changed before rendering".into());
    }
    Ok(())
}
