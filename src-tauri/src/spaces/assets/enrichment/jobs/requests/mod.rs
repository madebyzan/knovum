pub(crate) mod alignment;
pub(crate) mod diarization;

use super::job::EnrichmentJobType;
use crate::app_error::AppError;
use crate::spaces::db;
use sqlx::Row;
use tauri::AppHandle;

#[tauri::command]
pub async fn retry_enrichment_job(app: AppHandle, job_id: String) -> Result<(), AppError> {
    let pool = db::pool(&app).await?;
    let mut tx = db::begin_write(pool).await?;
    let row = sqlx::query("SELECT j.job_type,j.source_revision,n.data FROM spaces_enrichment_jobs j JOIN spaces_nodes n ON n.space_id=j.space_id AND n.id=j.node_id AND n.deleted_at IS NULL WHERE j.id=$1 AND j.status IN ('failed','cancelled')")
        .bind(&job_id).fetch_optional(&mut *tx).await.map_err(db::sql_error)?.ok_or("Only current failed or cancelled enrichment can be retried")?;
    let kind: EnrichmentJobType = serde_json::from_value(serde_json::json!(row
        .try_get::<String, _>("job_type")
        .map_err(db::sql_error)?))
    .map_err(db::json_error)?;
    let data: serde_json::Value =
        serde_json::from_str(&row.try_get::<String, _>("data").map_err(db::sql_error)?)
            .map_err(db::json_error)?;
    let revision: String = row.try_get("source_revision").map_err(db::sql_error)?;
    if super::revision::expected_revision(&data, kind)?.as_deref() != Some(revision.as_str()) {
        return Err("Enrichment source revision is obsolete".into());
    }
    if kind == EnrichmentJobType::TranscribeMedia && has_current_transcript(&data, &revision) {
        return Err(
            "Current media transcript already exists; retrying transcription would discard transcript edits"
                .into(),
        );
    }
    if kind == EnrichmentJobType::AlignMediaTranscript {
        let expected: i64 = sqlx::query_scalar(
            "SELECT expected_transcript_revision FROM spaces_media_alignment_requests WHERE job_id=$1",
        )
        .bind(&job_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(db::sql_error)?;
        if data["mediaKnowledge"]["transcript"]["revision"].as_u64() != u64::try_from(expected).ok()
        {
            return Err(
                "Transcript revision is obsolete; request alignment for the edited segments".into(),
            );
        }
    }
    if kind == EnrichmentJobType::DiarizeMediaTranscript {
        let expected: i64 = sqlx::query_scalar(
            "SELECT expected_transcript_revision FROM spaces_media_diarization_requests WHERE job_id=$1",
        )
        .bind(&job_id).fetch_one(&mut *tx).await.map_err(db::sql_error)?;
        if data["mediaKnowledge"]["transcript"]["revision"].as_u64() != u64::try_from(expected).ok()
        {
            return Err("Transcript revision is obsolete; request diarization again".into());
        }
    }
    let executed = sqlx::query("UPDATE spaces_enrichment_jobs SET status='pending',error=NULL,updated_at=$1 WHERE id=$2 AND status IN ('failed','cancelled')")
        .bind(crate::clock::unix_time_millis()?).bind(job_id).execute(&mut *tx).await.map_err(db::sql_error)?;
    db::require_one(
        executed,
        "Only failed or cancelled enrichment can be retried",
    )?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(())
}

#[tauri::command]
pub async fn request_media_transcription(
    app: AppHandle,
    space_id: String,
    node_id: String,
) -> Result<String, AppError> {
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
        return Err("Transcription requires an audio or video node".into());
    }
    let revision = super::revision::expected_revision(&data, EnrichmentJobType::TranscribeMedia)?
        .ok_or("Media source is not ready for transcription")?;
    if data["mediaAnalysis"]["sourceRevision"].as_str() != Some(&revision) {
        return Err("Transcription requires current media analysis".into());
    }
    if has_current_transcript(&data, &revision) {
        return Err(
            "Current media transcript already exists; automatic retranscription would discard transcript edits"
                .into(),
        );
    }
    let now = crate::clock::unix_time_millis()?;
    let id = uuid::Uuid::new_v4().to_string();
    let row = sqlx::query("INSERT INTO spaces_enrichment_jobs(id,space_id,node_id,job_type,status,source_revision,attempts,created_at,updated_at) VALUES($1,$2,$3,'transcribe_media','pending',$4,0,$5,$5) ON CONFLICT(space_id,node_id,job_type,source_revision) WHERE status IN ('pending','running','awaiting_configuration') DO UPDATE SET updated_at=excluded.updated_at RETURNING id")
        .bind(id).bind(space_id).bind(node_id).bind(revision).bind(now).fetch_one(&mut *tx).await.map_err(db::sql_error)?;
    let id: String = row.try_get("id").map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(id)
}

pub(super) fn has_current_transcript(data: &serde_json::Value, source_revision: &str) -> bool {
    let knowledge = &data["mediaKnowledge"];
    knowledge["schemaVersion"].as_u64() == Some(5)
        && knowledge["sourceRevision"].as_str() == Some(source_revision)
        && serde_json::from_value::<crate::media::transcript::MediaTranscript>(
            knowledge["transcript"].clone(),
        )
        .is_ok_and(|transcript| {
            !transcript.id.is_empty()
                && transcript.revision > 0
                && transcript.source_revision == source_revision
        })
}

#[cfg(test)]
mod tests;
