use std::collections::HashSet;

use sqlx::Row;
use tauri::AppHandle;

use crate::{app_error::AppError, spaces::db};

use super::super::job::EnrichmentJobType;

#[tauri::command]
pub async fn request_media_alignment(
    app: AppHandle,
    space_id: String,
    node_id: String,
    expected_revision: u32,
    mut segment_ids: Vec<String>,
) -> Result<String, AppError> {
    if expected_revision == 0 || segment_ids.is_empty() || segment_ids.len() > 8 {
        return Err(AppError::invalid(
            "Alignment requires a positive transcript revision and one to eight segments",
        ));
    }
    if segment_ids.iter().any(|id| id.trim().is_empty()) {
        return Err(AppError::invalid(
            "Alignment segment identifiers are required",
        ));
    }
    segment_ids.sort();
    if segment_ids.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(AppError::invalid(
            "Alignment segment identifiers must be unique",
        ));
    }

    let model = crate::local_resources::model_selection::speech_alignment(&app)?;
    let model_spec_id = model.alignment_spec()?;
    let pool = db::pool(&app).await?;
    let mut tx = db::begin_write(pool).await?;
    let raw: String = sqlx::query_scalar(
        "SELECT data FROM spaces_nodes WHERE space_id=$1 AND id=$2 AND deleted_at IS NULL",
    )
    .bind(&space_id)
    .bind(&node_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    let data: serde_json::Value = serde_json::from_str(&raw).map_err(db::json_error)?;
    if !matches!(data["kind"].as_str(), Some("audio" | "video")) {
        return Err(AppError::invalid(
            "Alignment requires an audio or video node",
        ));
    }
    let revision =
        super::super::revision::expected_revision(&data, EnrichmentJobType::AlignMediaTranscript)?
            .ok_or("Media source is not ready for alignment")?;
    let transcript: crate::media::transcript::MediaTranscript =
        serde_json::from_value(data["mediaKnowledge"]["transcript"].clone())
            .map_err(|error| AppError::invalid(format!("Media transcript is invalid: {error}")))?;
    if transcript.revision != expected_revision || transcript.source_revision != revision {
        return Err(AppError::invalid("Transcript revision is obsolete"));
    }
    crate::spaces::assets::transcription::alignment::validate_language(&transcript.language)?;
    let requested = segment_ids
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    let selected = transcript
        .segments
        .iter()
        .filter(|segment| requested.contains(segment.id.as_str()))
        .collect::<Vec<_>>();
    if selected.len() != segment_ids.len() {
        return Err(AppError::invalid("Alignment segment no longer exists"));
    }
    if selected
        .iter()
        .any(|segment| segment.text.trim().is_empty() || segment.end_ms <= segment.start_ms)
    {
        return Err(AppError::invalid("Alignment segment is invalid"));
    }
    let duration = selected.iter().try_fold(0_u64, |total, segment| {
        total.checked_add(segment.end_ms.saturating_sub(segment.start_ms))
    });
    if duration.is_none_or(|duration| duration > 300_000) {
        return Err(AppError::invalid(
            "Selected alignment audio must not exceed five minutes",
        ));
    }

    let segment_ids_json = serde_json::to_string(&segment_ids).map_err(db::json_error)?;
    if let Some(row) = sqlx::query(
        "SELECT j.id,r.model_spec_id,r.expected_transcript_revision,r.segment_ids FROM spaces_enrichment_jobs j \
         JOIN spaces_media_alignment_requests r ON r.job_id=j.id \
         WHERE j.space_id=$1 AND j.node_id=$2 AND j.job_type='align_media_transcript' \
           AND j.source_revision=$3 AND j.status IN ('pending','running','awaiting_configuration')",
    )
    .bind(&space_id)
    .bind(&node_id)
    .bind(&revision)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db::sql_error)?
    {
        let queued_revision: i64 = row
            .try_get("expected_transcript_revision")
            .map_err(db::sql_error)?;
        let queued_segments: String = row.try_get("segment_ids").map_err(db::sql_error)?;
        if queued_revision == i64::from(expected_revision) && queued_segments == segment_ids_json
            && row.try_get::<String, _>("model_spec_id").map_err(db::sql_error)? == model_spec_id {
            return Ok(row.try_get("id").map_err(db::sql_error)?);
        }
        return Err(AppError::invalid(
            "A different alignment request is already active for this transcript",
        ));
    }

    let now = crate::clock::unix_time_millis()?;
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO spaces_enrichment_jobs(id,space_id,node_id,job_type,status,source_revision,attempts,created_at,updated_at) VALUES($1,$2,$3,'align_media_transcript','pending',$4,0,$5,$5)")
        .bind(&id).bind(&space_id).bind(&node_id).bind(&revision).bind(now)
        .execute(&mut *tx).await.map_err(db::sql_error)?;
    sqlx::query("INSERT INTO spaces_media_alignment_requests(job_id,model_spec_id,expected_transcript_revision,segment_ids) VALUES($1,$2,$3,$4)")
        .bind(&id).bind(model_spec_id).bind(i64::from(expected_revision)).bind(segment_ids_json)
        .execute(&mut *tx).await.map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(id)
}
