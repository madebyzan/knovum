use sqlx::Row;
use tauri::AppHandle;

use crate::{app_error::AppError, media::transcript::MediaTranscript, spaces::db};

use super::super::job::EnrichmentJobType;

#[tauri::command]
pub async fn request_media_diarization(
    app: AppHandle,
    space_id: String,
    node_id: String,
    expected_revision: u32,
    num_speakers: Option<u16>,
) -> Result<String, AppError> {
    validate_bounds(expected_revision, num_speakers, num_speakers)?;
    let minimum_speakers = num_speakers;
    let maximum_speakers = num_speakers;
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
            "Diarization requires an audio or video node",
        ));
    }
    let source_revision = super::super::revision::expected_revision(
        &data,
        EnrichmentJobType::DiarizeMediaTranscript,
    )?
    .ok_or("Media source is not ready for diarization")?;
    let transcript: MediaTranscript =
        serde_json::from_value(data["mediaKnowledge"]["transcript"].clone())
            .map_err(|error| AppError::invalid(format!("Media transcript is invalid: {error}")))?;
    if transcript.revision != expected_revision || transcript.source_revision != source_revision {
        return Err(AppError::invalid("Transcript revision is obsolete"));
    }
    if transcript.duration_ms == 0 || transcript.duration_ms > 2 * 60 * 60 * 1_000 {
        return Err(AppError::invalid(
            "Diarization media must not exceed two hours",
        ));
    }
    if transcript.segments.iter().any(|segment| {
        segment.end_ms <= segment.start_ms || segment.end_ms > transcript.duration_ms
    }) {
        return Err(AppError::invalid("Transcript segment timing is invalid"));
    }
    if let Some(row) = sqlx::query(
        "SELECT j.id,r.expected_transcript_revision,r.minimum_speakers,r.maximum_speakers \
         FROM spaces_enrichment_jobs j JOIN spaces_media_diarization_requests r ON r.job_id=j.id \
         WHERE j.space_id=$1 AND j.node_id=$2 AND j.job_type='diarize_media_transcript' \
           AND j.source_revision=$3 AND j.status IN ('pending','running','awaiting_configuration')",
    )
    .bind(&space_id)
    .bind(&node_id)
    .bind(&source_revision)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db::sql_error)?
    {
        let queued_revision: i64 = row
            .try_get("expected_transcript_revision")
            .map_err(db::sql_error)?;
        let queued_minimum: Option<i64> = row.try_get("minimum_speakers").map_err(db::sql_error)?;
        let queued_maximum: Option<i64> = row.try_get("maximum_speakers").map_err(db::sql_error)?;
        if queued_revision == i64::from(expected_revision)
            && queued_minimum == minimum_speakers.map(i64::from)
            && queued_maximum == maximum_speakers.map(i64::from)
        {
            return Ok(row.try_get("id").map_err(db::sql_error)?);
        }
        return Err(AppError::invalid(
            "A different diarization request is already active",
        ));
    }
    let model = crate::local_resources::model_selection::speech_diarization(&app)?;
    let model_spec = model.diarization_variant()?.spec_id();
    let now = crate::clock::unix_time_millis()?;
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO spaces_enrichment_jobs(id,space_id,node_id,job_type,status,source_revision,attempts,created_at,updated_at) VALUES($1,$2,$3,'diarize_media_transcript','pending',$4,0,$5,$5)")
        .bind(&id).bind(&space_id).bind(&node_id).bind(&source_revision).bind(now)
        .execute(&mut *tx).await.map_err(db::sql_error)?;
    sqlx::query("INSERT INTO spaces_media_diarization_requests(job_id,expected_transcript_revision,minimum_speakers,maximum_speakers,model_spec_id) VALUES($1,$2,$3,$4,$5)")
        .bind(&id).bind(i64::from(expected_revision)).bind(minimum_speakers.map(i64::from))
        .bind(maximum_speakers.map(i64::from)).bind(model_spec).execute(&mut *tx).await.map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(id)
}

fn validate_bounds(
    revision: u32,
    minimum: Option<u16>,
    maximum: Option<u16>,
) -> Result<(), AppError> {
    if revision == 0
        || minimum.is_some_and(|value| value == 0 || value > 32)
        || maximum.is_some_and(|value| value == 0 || value > 32)
        || matches!((minimum, maximum), (Some(minimum), Some(maximum)) if minimum > maximum)
    {
        return Err(AppError::invalid(
            "Speaker bounds must be within one and 32",
        ));
    }
    Ok(())
}
