use serde_json::Value;
use sha2::{Digest, Sha256};
use tauri::AppHandle;

use super::{GenerateVideoRequest, VideoContent, VideoOutput};
use crate::{
    foundation::app_error::AppError,
    spaces::{
        db,
        derivation::{pending_output, source_inputs},
        knowledge::model::revision,
    },
};

pub async fn complete(
    app: &AppHandle,
    request: &GenerateVideoRequest,
    expected_path: &str,
    generated: VideoContent,
    authority: &super::super::action_execution::ExecutionAuthority,
    cancellation: &tokio_util::sync::CancellationToken,
) -> Result<VideoOutput, AppError> {
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    authority.require_active(&mut *tx).await?;
    let encoded: String = sqlx::query_scalar(
        "SELECT n.data FROM spaces_knowledge_derivations d JOIN spaces_nodes n \
         ON n.id=d.output_node_id AND n.space_id=d.space_id \
         WHERE d.id=$1 AND d.space_id=$2 AND d.status='running' AND n.deleted_at IS NULL",
    )
    .bind(&request.id)
    .bind(&request.space_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db::sql_error)?
    .ok_or("Generated video is no longer running")?;
    let mut data: Value = serde_json::from_str(&encoded).map_err(db::json_error)?;
    pending_output::validate_unpublished_asset(&data)?;
    if data.get("assetPath").and_then(Value::as_str) != Some(expected_path) {
        return Err("Generated video output path changed while rendering".into());
    }
    source_inputs::pin(&mut tx, &request.space_id, &request.inputs).await?;
    let now = crate::foundation::clock::unix_time_millis()?;
    let cover = crate::spaces::previews::generate_for_published_asset_off_thread(
        app,
        std::path::Path::new(expected_path),
        cancellation,
    )
    .await?;
    let object = data
        .as_object_mut()
        .ok_or("Generated video node data is invalid")?;
    crate::spaces::assets::published_asset::PublishedAsset {
        path: expected_path,
        revision: format!("sha256:{}", hex::encode(Sha256::digest(&generated.bytes))),
        size: Some(generated.bytes.len() as u64),
        cover: cover.as_ref(),
    }
    .settle_onto(object, now)?;
    revision::apply(&mut data)?;
    crate::spaces::knowledge::validation::validate_node(&mut tx, &request.space_id, "video", &data)
        .await?;
    crate::spaces::derivation::persist_output_node(
        &mut tx,
        &request.output_node_id,
        &data,
        now,
        crate::spaces::assets::enrichment::jobs::intent::selection(app)?.as_ref(),
    )
    .await?;
    sqlx::query("INSERT INTO spaces_generated_videos(derivation_id,prompt,aspect_ratio,duration_seconds,execution_source,operation_id) VALUES($1,$2,$3,$4,$5,$6)")
        .bind(&request.id).bind(&request.prompt).bind(request.aspect_ratio.as_str()).bind(request.duration_seconds)
        .bind(sqlx::types::Json(&generated.execution_source)).bind(generated.operation_id)
        .execute(&mut *tx).await.map_err(db::sql_error)?;
    sqlx::query("UPDATE spaces_knowledge_derivations SET status='completed',completed_at=$1,updated_at=$1 WHERE id=$2")
        .bind(now).bind(&request.id).execute(&mut *tx).await.map_err(db::sql_error)?;
    if cancellation.is_cancelled() {
        return Err("Video generation was cancelled".into());
    }
    tx.commit().await.map_err(db::sql_error)?;
    Ok(VideoOutput {
        node_id: request.output_node_id.clone(),
        data,
    })
}
