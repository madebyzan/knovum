use serde_json::Value;
use tauri::AppHandle;

use super::{persistence::VideoSubtitleSource, CreateSubtitledVideoRequest};
use crate::spaces::derivation::inputs;
use crate::spaces::{db, knowledge::contracts};

pub async fn start(
    app: &AppHandle,
    request: &CreateSubtitledVideoRequest,
) -> Result<VideoSubtitleSource, String> {
    validate_job_id(&request.id)?;
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    contracts::validate_same_space_node(
        &mut tx,
        &request.space_id,
        &request.source_node_id,
        &request.output_node_id,
    )
    .await?;
    contracts::validate_same_space_node(
        &mut tx,
        &request.space_id,
        &request.subtitle_node_id,
        &request.output_node_id,
    )
    .await?;
    let video = node_data(&mut tx, &request.space_id, &request.source_node_id, "video").await?;
    validate_video(&video, &request.source_revision)?;
    let subtitle = node_data(
        &mut tx,
        &request.space_id,
        &request.subtitle_node_id,
        "subtitle",
    )
    .await?;
    validate_subtitle(&subtitle, request)?;
    let source = VideoSubtitleSource {
        path: required_string(&video, "assetPath")?.into(),
        srt: super::subtitle_text(&subtitle)?,
    };
    let now = crate::clock::unix_time_millis()?;
    sqlx::query("INSERT INTO spaces_knowledge_derivations(id,space_id,output_node_id,action_kind,action_version,engine,status,created_at,updated_at) VALUES($1,$2,$3,'render-subtitled-video',1,'ffmpeg:libass+platform-h264','running',$4,$4)")
        .bind(&request.id).bind(&request.space_id).bind(&request.output_node_id).bind(now)
        .execute(&mut *tx).await.map_err(db::sql_error)?;
    inputs::insert(
        &mut tx,
        &request.id,
        &[
            inputs::DerivationInput {
                node_id: &request.source_node_id,
                role: "primary",
                source_revision: &request.source_revision,
                revision_basis: inputs::RevisionBasis::Asset,
            },
            inputs::DerivationInput {
                node_id: &request.subtitle_node_id,
                role: "source",
                source_revision: &request.subtitle_revision,
                revision_basis: inputs::RevisionBasis::Knowledge,
            },
        ],
    )
    .await?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(source)
}

async fn node_data(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    id: &str,
    kind: &str,
) -> Result<Value, String> {
    let encoded: String = sqlx::query_scalar("SELECT data FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND kind=$3 AND deleted_at IS NULL")
        .bind(id).bind(space_id).bind(kind).fetch_optional(&mut **tx).await.map_err(db::sql_error)?
        .ok_or_else(|| format!("{kind} source is unavailable"))?;
    serde_json::from_str(&encoded).map_err(db::json_error)
}

fn validate_video(value: &Value, expected: &str) -> Result<(), String> {
    if value.get("sourceRevision").and_then(Value::as_str) != Some(expected)
        || value
            .pointer("/mediaKnowledge/sourceRevision")
            .and_then(Value::as_str)
            != Some(expected)
        || value
            .pointer("/mediaKnowledge/schemaVersion")
            .and_then(Value::as_u64)
            != Some(4)
    {
        return Err("Analyze the current video before creating subtitles".into());
    }
    Ok(())
}

fn validate_subtitle(value: &Value, request: &CreateSubtitledVideoRequest) -> Result<(), String> {
    if value.get("knowledgeRevision").and_then(Value::as_str)
        != Some(request.subtitle_revision.as_str())
        || value.get("sourceVideoNodeId").and_then(Value::as_str)
            != Some(request.source_node_id.as_str())
        || value.get("sourceVideoRevision").and_then(Value::as_str)
            != Some(request.source_revision.as_str())
    {
        return Err("Subtitles do not match the selected video revision".into());
    }
    Ok(())
}

fn validate_job_id(id: &str) -> Result<(), String> {
    if id.is_empty()
        || id.len() > 64
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        return Err("Subtitle derivation job id is invalid".into());
    }
    Ok(())
}

fn required_string<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
        .ok_or_else(|| format!("Video {key} is unavailable"))
}
