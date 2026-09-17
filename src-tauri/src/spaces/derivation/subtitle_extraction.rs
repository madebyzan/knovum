use serde::Deserialize;
use serde_json::Value;
use tauri::AppHandle;

use super::{inputs, relations, DerivationRelation};
use crate::spaces::{db, knowledge::contracts};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateSubtitleResourceRequest {
    id: String,
    space_id: String,
    source_node_id: String,
    output_node_id: String,
    source_revision: String,
}

pub async fn create(
    app: &AppHandle,
    request: CreateSubtitleResourceRequest,
) -> Result<DerivationRelation, String> {
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    contracts::validate_same_space_node(
        &mut tx,
        &request.space_id,
        &request.source_node_id,
        &request.output_node_id,
    )
    .await?;
    let source: String = sqlx::query_scalar("SELECT data FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND kind='video' AND deleted_at IS NULL")
        .bind(&request.source_node_id).bind(&request.space_id).fetch_optional(&mut *tx).await.map_err(db::sql_error)?
        .ok_or("Subtitle source video is unavailable")?;
    let output: String = sqlx::query_scalar("SELECT data FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND kind='subtitle' AND deleted_at IS NULL")
        .bind(&request.output_node_id).bind(&request.space_id).fetch_optional(&mut *tx).await.map_err(db::sql_error)?
        .ok_or("Subtitle output is unavailable")?;
    let source: Value = serde_json::from_str(&source).map_err(db::json_error)?;
    let output: Value = serde_json::from_str(&output).map_err(db::json_error)?;
    validate(&source, &output, &request)?;
    let now = crate::clock::unix_time_millis()?;
    sqlx::query("INSERT INTO spaces_knowledge_derivations(id,space_id,output_node_id,action_kind,action_version,engine,status,created_at,updated_at,completed_at) VALUES($1,$2,$3,'video-subtitle-extraction',1,'whisper-token-timing:1','completed',$4,$4,$4)")
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
    let relation = relations::derives_from(
        &request.space_id,
        &request.output_node_id,
        &request.source_node_id,
        "Subtitles extracted from video transcript timing",
        now,
    );
    relations::insert(&mut tx, &relation).await?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(relation)
}

fn validate(
    source: &Value,
    output: &Value,
    request: &CreateSubtitleResourceRequest,
) -> Result<(), String> {
    if source.get("sourceRevision").and_then(Value::as_str)
        != Some(request.source_revision.as_str())
        || source
            .pointer("/mediaKnowledge/sourceRevision")
            .and_then(Value::as_str)
            != Some(request.source_revision.as_str())
        || output.get("sourceVideoNodeId").and_then(Value::as_str)
            != Some(request.source_node_id.as_str())
        || output.get("sourceVideoRevision").and_then(Value::as_str)
            != Some(request.source_revision.as_str())
    {
        return Err("Video knowledge changed before subtitle extraction completed".into());
    }
    Ok(())
}
