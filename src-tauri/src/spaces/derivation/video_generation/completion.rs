use serde_json::Value;
use sha2::{Digest, Sha256};
use tauri::AppHandle;

use super::{GenerateVideoRequest, VideoOutput};
use crate::ai::capabilities::video::GeneratedVideo;
use crate::spaces::derivation::{pending_output, relations, source_validation};
use crate::spaces::{
    db,
    knowledge::{contracts, indexing::revision},
};

pub async fn complete(
    app: &AppHandle,
    request: &GenerateVideoRequest,
    expected_path: &str,
    generated: GeneratedVideo,
    authority: &super::super::action_execution::ExecutionAuthority,
) -> Result<VideoOutput, String> {
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
    for input in &request.inputs {
        source_validation::validate(
            &mut tx,
            &request.space_id,
            &input.node_id,
            &input.source_revision,
            super::preparation::input_basis(&input.role)?,
        )
        .await?;
    }
    let now = crate::clock::unix_time_millis()?;
    let object = data
        .as_object_mut()
        .ok_or("Generated video node data is invalid")?;
    object.insert(
        "sourceRevision".into(),
        format!("sha256:{:x}", Sha256::digest(&generated.bytes)).into(),
    );
    object.insert("size".into(), (generated.bytes.len() as u64).into());
    object.insert("modifiedAt".into(), now.into());
    object.remove("processingState");
    revision::apply(&mut data)?;
    contracts::validate_node(
        &mut tx,
        &request.space_id,
        &request.output_node_id,
        "video",
        &data,
    )
    .await?;
    crate::spaces::derivation::persist_output_node(&mut tx, &request.output_node_id, &data, now)
        .await?;
    let mut output_relations = Vec::with_capacity(request.inputs.len());
    for input in &request.inputs {
        let relation = relations::derives_from(
            &request.space_id,
            &request.output_node_id,
            &input.node_id,
            "Video generated from selected knowledge",
            now,
        );
        relations::insert(&mut tx, &relation).await?;
        output_relations.push(relation);
    }
    sqlx::query("INSERT INTO spaces_generated_videos(derivation_id,prompt,aspect_ratio,duration_seconds,provider,model,operation_id) VALUES($1,$2,$3,$4,$5,$6,$7)")
        .bind(&request.id).bind(&request.prompt).bind(&request.aspect_ratio).bind(request.duration_seconds)
        .bind(generated.provider).bind(generated.model).bind(generated.operation_id)
        .execute(&mut *tx).await.map_err(db::sql_error)?;
    sqlx::query("UPDATE spaces_knowledge_derivations SET status='completed',completed_at=$1,updated_at=$1 WHERE id=$2")
        .bind(now).bind(&request.id).execute(&mut *tx).await.map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(VideoOutput {
        node_id: request.output_node_id.clone(),
        data,
        relations: output_relations,
    })
}
