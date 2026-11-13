pub(crate) mod alignment;
pub(crate) mod diarization;

use super::job::EnrichmentJobType;
use crate::ai::tasks::TaskTargetSelection;
use crate::foundation::app_error::AppError;
use crate::local_resources::model_catalog::LocalModelId;
use crate::spaces::db;
use serde::{de::IntoDeserializer, Deserialize};
use sqlx::Row;
use tauri::AppHandle;

#[tauri::command]
pub async fn retry_enrichment_job(
    app: AppHandle,
    job_id: String,
    target: Option<TaskTargetSelection>,
) -> Result<(), AppError> {
    let pool = db::pool(&app).await?;
    let mut tx = db::begin_write(pool).await?;
    let row = sqlx::query("SELECT j.space_id,j.node_id,j.job_type,j.source_revision,n.data FROM spaces_enrichment_jobs j JOIN spaces_nodes n ON n.space_id=j.space_id AND n.id=j.node_id AND n.deleted_at IS NULL WHERE j.id=$1 AND j.status IN ('failed','cancelled')")
        .bind(&job_id).fetch_optional(&mut *tx).await.map_err(db::sql_error)?.ok_or("Only current failed or cancelled enrichment can be retried")?;
    let space_id: String = row.try_get("space_id").map_err(db::sql_error)?;
    let node_id: String = row.try_get("node_id").map_err(db::sql_error)?;
    let kind = EnrichmentJobType::deserialize(
        row.try_get::<String, _>("job_type")
            .map_err(db::sql_error)?
            .into_deserializer(),
    )
    .map_err(db::json_error)?;
    let data: serde_json::Value =
        serde_json::from_str(&row.try_get::<String, _>("data").map_err(db::sql_error)?)
            .map_err(db::json_error)?;
    let revision: String = row.try_get("source_revision").map_err(db::sql_error)?;
    if super::revision::expected_revision(&data, kind)?.as_deref() != Some(revision.as_str()) {
        return Err("Enrichment source revision is obsolete".into());
    }
    if kind == EnrichmentJobType::TranscribeMedia {
        ensure_no_active_transcript_consumers(&mut tx, &space_id, &node_id).await?;
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
    let replacement = match kind {
        EnrichmentJobType::AnalyzeVisual => target,
        _ => None,
    };
    let executed = sqlx::query("UPDATE spaces_enrichment_jobs SET status='pending',error=NULL,execution_target=COALESCE($1,execution_target),updated_at=$2 WHERE id=$3 AND status IN ('failed','cancelled')")
        .bind(replacement.map(sqlx::types::Json)).bind(crate::foundation::clock::unix_time_millis()?).bind(job_id).execute(&mut *tx).await.map_err(db::sql_error)?;
    db::require_one(
        executed,
        "Only failed or cancelled enrichment can be retried",
    )?;
    tx.commit().await.map_err(db::sql_error)?;
    crate::spaces::runs::announce_change();
    Ok(())
}

#[tauri::command]
pub async fn request_media_transcription(
    app: AppHandle,
    space_id: String,
    node_id: String,
    model: LocalModelId,
) -> Result<String, AppError> {
    if !model.is_transcription() {
        return Err(AppError::invalid(
            "Transcription requires a speech recognition model",
        ));
    }
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
    ensure_no_active_transcript_consumers(&mut tx, &space_id, &node_id).await?;
    let id = enqueue_manual_job(
        &mut tx,
        &space_id,
        &node_id,
        EnrichmentJobType::TranscribeMedia,
        &revision,
        None,
    )
    .await?;
    sqlx::query(
        "INSERT INTO spaces_media_transcription_requests(job_id,model_spec_id) VALUES($1,$2) \
         ON CONFLICT(job_id) DO NOTHING",
    )
    .bind(&id)
    .bind(sqlx::types::Json(model))
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)?;
    crate::spaces::runs::announce_change();
    Ok(id)
}

#[tauri::command]
pub async fn request_visual_analysis(
    app: AppHandle,
    space_id: String,
    node_id: String,
    target: TaskTargetSelection,
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
    let revision = super::revision::expected_revision(&data, EnrichmentJobType::AnalyzeVisual)?
        .ok_or("Visual source is not ready for analysis")?;
    let id = enqueue_manual_job(
        &mut tx,
        &space_id,
        &node_id,
        EnrichmentJobType::AnalyzeVisual,
        &revision,
        Some(target),
    )
    .await?;
    tx.commit().await.map_err(db::sql_error)?;
    crate::spaces::runs::announce_change();
    Ok(id)
}

async fn enqueue_manual_job(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    node_id: &str,
    job_type: EnrichmentJobType,
    revision: &str,
    target: Option<crate::ai::tasks::TaskTargetSelection>,
) -> Result<String, AppError> {
    let now = crate::foundation::clock::unix_time_millis()?;
    let id = uuid::Uuid::new_v4().to_string();
    let row = sqlx::query(
        "INSERT INTO spaces_enrichment_jobs(id,space_id,node_id,job_type,status,source_revision,execution_target,attempts,created_at,updated_at) \
         VALUES($1,$2,$3,$4,'pending',$5,$6,0,$7,$7) \
         ON CONFLICT(space_id,node_id,job_type,source_revision) WHERE status IN ('pending','running','awaiting_configuration') \
         DO UPDATE SET \
           status=CASE WHEN spaces_enrichment_jobs.status='awaiting_configuration' THEN 'pending' ELSE spaces_enrichment_jobs.status END, \
           error=CASE WHEN spaces_enrichment_jobs.status='awaiting_configuration' THEN NULL ELSE spaces_enrichment_jobs.error END, \
           execution_target=excluded.execution_target, \
           updated_at=excluded.updated_at \
         RETURNING id",
    )
    .bind(id)
    .bind(space_id)
    .bind(node_id)
    .bind(job_type.as_str())
    .bind(revision)
    .bind(target.map(sqlx::types::Json))
    .bind(now)
    .fetch_one(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    Ok(row.try_get("id").map_err(db::sql_error)?)
}

async fn ensure_no_active_transcript_consumers(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    node_id: &str,
) -> Result<(), AppError> {
    let active: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM spaces_enrichment_jobs \
         WHERE space_id=$1 AND node_id=$2 \
           AND job_type IN ('align_media_transcript','diarize_media_transcript') \
           AND status IN ('pending','running','awaiting_configuration'))",
    )
    .bind(space_id)
    .bind(node_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    if active {
        return Err(AppError::invalid(
            "Cannot retranscribe while transcript alignment or diarization is active",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
