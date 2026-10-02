use serde_json::Value;
use sqlx::SqlitePool;

use super::{
    preparation::{PreparedSubtitles, SubtitleVideoSource},
    RenderSubtitledVideoRequest,
};
use crate::spaces::derivation::inputs;
use crate::spaces::derivation::pending_output;
use crate::spaces::{db, knowledge::contracts};

pub async fn start(
    pool: &SqlitePool,
    request: &RenderSubtitledVideoRequest,
    authority: &super::super::action_execution::ExecutionAuthority,
) -> Result<SubtitleVideoSource, String> {
    validate_job_id(&request.id)?;
    let mut tx = db::begin_write(pool).await?;
    authority.require_active(&mut *tx).await?;
    contracts::validate_same_space_node(
        &mut tx,
        &request.space_id,
        &request.source_node_id,
        &request.output_node_id,
    )
    .await?;
    let video =
        load_node_content(&mut tx, &request.space_id, &request.source_node_id, "video").await?;
    if video.get("sourceRevision").and_then(Value::as_str) != Some(request.source_revision.as_str())
    {
        return Err("Video source changed before subtitle creation".into());
    }
    let output =
        load_node_content(&mut tx, &request.space_id, &request.output_node_id, "video").await?;
    pending_output::validate_separate_asset(&output, required_string(&video, "assetPath")?)?;
    let subtitles = automatic_subtitles(&video, &request.source_revision)?;
    let captured_inputs = [inputs::DerivationInput {
        node_id: &request.source_node_id,
        role: "primary",
        source_revision: &request.knowledge_revision,
        revision_basis: inputs::RevisionBasis::Knowledge,
    }];
    let engine = match &subtitles {
        PreparedSubtitles::Ready(_) => "ffmpeg:libass+platform-h264",
        PreparedSubtitles::Recognize => "whisper.cpp+ffmpeg:libass+platform-h264",
    };
    let source = SubtitleVideoSource {
        path: required_string(&video, "assetPath")?.into(),
        subtitles,
    };
    let now = crate::clock::unix_time_millis()?;
    sqlx::query("INSERT INTO spaces_knowledge_derivations(id,space_id,output_node_id,action_kind,action_version,engine,status,created_at,updated_at) VALUES($1,$2,$3,$4,1,$5,'running',$6,$6)")
        .bind(&request.id).bind(&request.space_id).bind(&request.output_node_id).bind("add-video-subtitles").bind(engine).bind(now)
        .execute(&mut *tx).await.map_err(db::sql_error)?;
    inputs::insert(&mut tx, &request.id, &captured_inputs).await?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(source)
}

async fn load_node_content(
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

pub(super) fn automatic_subtitles(
    video: &Value,
    source_revision: &str,
) -> Result<PreparedSubtitles, String> {
    let Some(knowledge) = video.get("mediaKnowledge") else {
        return Ok(PreparedSubtitles::Recognize);
    };
    if knowledge.get("sourceRevision").and_then(Value::as_str) != Some(source_revision)
        || knowledge.get("schemaVersion").and_then(Value::as_u64) != Some(4)
    {
        return Err("Video knowledge does not match the current source".into());
    }
    match knowledge.pointer("/transcript/subtitleCues") {
        Some(cues) => {
            let duration = knowledge
                .pointer("/transcript/durationMs")
                .and_then(Value::as_u64)
                .ok_or("Video transcript duration is unavailable")?;
            let cues: Vec<crate::media::transcript::SubtitleCue> =
                serde_json::from_value(cues.clone())
                    .map_err(|error| format!("Subtitle cues are invalid: {error}"))?;
            Ok(PreparedSubtitles::Ready(super::srt::from_cues(
                &cues, duration,
            )?))
        }
        None => Ok(PreparedSubtitles::Recognize),
    }
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
