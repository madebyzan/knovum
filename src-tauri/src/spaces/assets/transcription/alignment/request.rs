use std::collections::HashSet;

use sqlx::Row;
use tauri::AppHandle;

use crate::{
    ai::contracts::speech_wire::AlignmentTextSegment, media::transcript::MediaTranscript,
    spaces::db,
};

pub(super) struct AlignmentRequest {
    pub(super) model: crate::local_resources::model_catalog::LocalModelId,
    pub(super) expected_revision: u32,
    pub(super) segment_ids: Vec<String>,
}

pub(super) async fn load(app: &AppHandle, job_id: &str) -> Result<AlignmentRequest, String> {
    let row = sqlx::query(
        "SELECT model_spec_id,expected_transcript_revision,segment_ids FROM spaces_media_alignment_requests WHERE job_id=$1",
    )
    .bind(job_id)
    .fetch_optional(db::pool(app).await?)
    .await
    .map_err(db::sql_error)?
    .ok_or("Alignment request payload is missing")?;
    let revision: i64 = row
        .try_get("expected_transcript_revision")
        .map_err(db::sql_error)?;
    let raw: String = row.try_get("segment_ids").map_err(db::sql_error)?;
    let segment_ids: Vec<String> = serde_json::from_str(&raw).map_err(db::json_error)?;
    Ok(AlignmentRequest {
        model: crate::local_resources::model_catalog::LocalModelId::from_alignment_spec(
            &row.try_get::<String, _>("model_spec_id")
                .map_err(db::sql_error)?,
        )?,
        expected_revision: u32::try_from(revision).map_err(|error| error.to_string())?,
        segment_ids,
    })
}

pub(super) fn segments(
    transcript: &MediaTranscript,
    request: &AlignmentRequest,
) -> Result<(Vec<AlignmentTextSegment>, u64), String> {
    if request.segment_ids.is_empty()
        || request.segment_ids.len() > 8
        || request.segment_ids.iter().any(|id| id.trim().is_empty())
    {
        return Err("Alignment request payload is invalid".into());
    }
    let ids = request.segment_ids.iter().collect::<HashSet<_>>();
    if ids.len() != request.segment_ids.len() {
        return Err("Alignment segment identifiers must be unique".into());
    }
    let selected = transcript
        .segments
        .iter()
        .filter(|segment| ids.contains(&segment.id))
        .map(|segment| AlignmentTextSegment {
            segment_id: segment.id.clone(),
            start_ms: segment.start_ms,
            end_ms: segment.end_ms,
            text: segment.text.clone(),
        })
        .collect::<Vec<_>>();
    if selected.len() != request.segment_ids.len() {
        return Err("Alignment segment no longer exists".into());
    }
    if selected
        .iter()
        .any(|segment| segment.text.trim().is_empty() || segment.end_ms <= segment.start_ms)
    {
        return Err("Alignment segment is invalid".into());
    }
    let duration = selected.iter().try_fold(0_u64, |total, segment| {
        total.checked_add(segment.end_ms - segment.start_ms)
    });
    match duration {
        Some(duration) if duration <= 300_000 => Ok((selected, duration)),
        _ => Err("Selected alignment audio must not exceed five minutes".into()),
    }
}
