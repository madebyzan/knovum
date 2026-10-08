use sqlx::{Row, SqlitePool};
use std::path::Path;

use super::{VoiceCandidate, VoiceCandidateStatus, VoiceCandidateStepConfiguration, VoiceModel};
use crate::{
    ai::contracts::speech_wire::AudioFileReference,
    spaces::speech::contracts::SpeechAudioOutput,
    spaces::{db, runs::AttemptClaim},
};

pub(crate) struct StoredVoiceCandidate {
    pub candidate: VoiceCandidate,
    pub reference_asset_path: String,
    pub reference_sha256: String,
    pub reference_byte_length: u64,
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn persist_candidate(
    pool: &SqlitePool,
    claim: &AttemptClaim,
    configuration: &VoiceCandidateStepConfiguration,
    source_kind: &str,
    design_description: Option<&str>,
    reference_text: Option<&str>,
    preview_path: &Path,
    preview_sha256: &str,
    preview_sample_rate: u32,
    preview_sample_count: u64,
    reference_path: &Path,
    reference: &AudioFileReference,
) -> Result<(), String> {
    let request = &configuration.request;
    let now = crate::clock::unix_time_millis()?;
    let preview_sample_count = i64::try_from(preview_sample_count)
        .map_err(|_| "Voice candidate sample count exceeds storage range")?;
    let reference_byte_length = i64::try_from(reference.byte_length)
        .map_err(|_| "Voice candidate reference length exceeds storage range")?;
    let input_byte_length = configuration
        .reference_source
        .as_ref()
        .map(|source| i64::try_from(source.byte_length))
        .transpose()
        .map_err(|_| "Voice candidate input length exceeds storage range")?;
    let authorization = serde_json::to_string(&serde_json::json!({
        "declared": request.authorized,
        "declaredAt": now,
        "referenceTranscriptConfirmed": reference_text.is_some(),
    }))
    .map_err(db::json_error)?;
    let mut tx = db::begin_write(pool).await?;
    let inserted = sqlx::query(
        "INSERT INTO spaces_voice_candidates( \
           id,space_id,run_id,source_kind,language,sample_text,design_description,reference_text,input_source_revision,input_byte_length, \
           reference_asset_path,reference_sha256,reference_byte_length,preview_asset_path,preview_sha256, \
           preview_sample_rate,preview_sample_count,model_spec_id,seed,synthesis_config,authorization,status,created_at) \
         SELECT ?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?, 'ready',? WHERE EXISTS( \
           SELECT 1 FROM spaces_run_attempts a JOIN spaces_runs r ON r.id=a.run_id \
           WHERE a.id=? AND a.run_id=? AND a.step_instance_id=? AND a.state='executing' \
             AND a.execution_epoch=? AND a.lease_until>? AND r.execution_epoch=a.execution_epoch \
             AND r.cancel_requested_at IS NULL AND r.completed_at IS NULL)",
    )
    .bind(&claim.run_id)
    .bind(&claim.home_space_id)
    .bind(&claim.run_id)
    .bind(source_kind)
    .bind(&request.language)
    .bind(request.text.trim())
    .bind(design_description)
    .bind(reference_text)
    .bind(match &request.source {
        super::VoiceCandidateSource::Reference { source_revision, .. } => Some(source_revision.as_str()),
        super::VoiceCandidateSource::Designed { .. } => None,
    })
    .bind(input_byte_length)
    .bind(reference_path.to_string_lossy().as_ref())
    .bind(&reference.sha256)
    .bind(reference_byte_length)
    .bind(preview_path.to_string_lossy().as_ref())
    .bind(preview_sha256)
    .bind(i64::from(preview_sample_rate))
    .bind(preview_sample_count)
    .bind(request.model.specification())
    .bind(i64::from(request.seed))
    .bind(serde_json::json!({"speed":1.0,"sampleRate":48000,"seed":request.seed}).to_string())
    .bind(authorization)
    .bind(now)
    .bind(&claim.attempt_id)
    .bind(&claim.run_id)
    .bind(&claim.step_instance_id)
    .bind(claim.execution_epoch)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    db::require_one(
        inserted,
        "attempt-stale: voice candidate publication lease is no longer active",
    )?;
    super::super::asset_publication::remove_attempt_in_transaction(
        &mut tx,
        claim,
        preview_path,
        preview_sha256,
    )
    .await?;
    super::super::asset_publication::remove_attempt_in_transaction(
        &mut tx,
        claim,
        reference_path,
        &reference.sha256,
    )
    .await?;
    tx.commit().await.map_err(db::sql_error)
}

pub(crate) async fn load_candidate(
    pool: &SqlitePool,
    candidate_id: &str,
    space_id: &str,
) -> Result<VoiceCandidate, String> {
    Ok(load_record(pool, candidate_id, space_id).await?.candidate)
}

pub(crate) async fn load_record(
    pool: &SqlitePool,
    candidate_id: &str,
    space_id: &str,
) -> Result<StoredVoiceCandidate, String> {
    let row = sqlx::query(
        "SELECT id,run_id,space_id,source_kind,language,sample_text,design_description,reference_text, \
         reference_asset_path,reference_sha256,reference_byte_length,preview_asset_path,preview_sha256, \
         preview_sample_rate,preview_sample_count,model_spec_id,seed,status,auditioned_at,selected_voice_profile_id, \
         selected_voice_revision_id,selected_at,rejected_at,created_at \
         FROM spaces_voice_candidates WHERE id=? AND space_id=?",
    )
    .bind(candidate_id)
    .bind(space_id)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?
    .ok_or("Voice candidate is unavailable in this Space")?;
    decode(row)
}

pub(crate) async fn load_record_optional(
    pool: &SqlitePool,
    candidate_id: &str,
    space_id: &str,
) -> Result<Option<StoredVoiceCandidate>, String> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM spaces_voice_candidates WHERE id=? AND space_id=?)",
    )
    .bind(candidate_id)
    .bind(space_id)
    .fetch_one(pool)
    .await
    .map_err(db::sql_error)?;
    if exists {
        Ok(Some(load_record(pool, candidate_id, space_id).await?))
    } else {
        Ok(None)
    }
}

pub(crate) async fn list_ready(
    pool: &SqlitePool,
    space_id: &str,
) -> Result<Vec<VoiceCandidate>, String> {
    let rows = sqlx::query("SELECT * FROM spaces_voice_candidates WHERE space_id=? AND status='ready' ORDER BY created_at DESC,id DESC LIMIT 16")
        .bind(space_id).fetch_all(pool).await.map_err(db::sql_error)?;
    rows.into_iter()
        .map(|row| decode(row).map(|record| record.candidate))
        .collect()
}

pub(crate) async fn mark_auditioned(
    pool: &SqlitePool,
    candidate_id: &str,
    space_id: &str,
    now: i64,
) -> Result<(), String> {
    let changed = sqlx::query(
        "UPDATE spaces_voice_candidates SET auditioned_at=COALESCE(auditioned_at,?) \
         WHERE id=? AND space_id=? AND status='ready'",
    )
    .bind(now)
    .bind(candidate_id)
    .bind(space_id)
    .execute(pool)
    .await
    .map_err(db::sql_error)?;
    db::require_one(
        changed,
        "Voice candidate is no longer available for audition",
    )
}

pub(crate) async fn reject_candidate(
    pool: &SqlitePool,
    candidate_id: &str,
    space_id: &str,
) -> Result<(), String> {
    let changed = sqlx::query(
        "UPDATE spaces_voice_candidates SET status='rejected',rejected_at=? \
         WHERE id=? AND space_id=? AND status='ready'",
    )
    .bind(crate::clock::unix_time_millis()?)
    .bind(candidate_id)
    .bind(space_id)
    .execute(pool)
    .await
    .map_err(db::sql_error)?;
    db::require_one(
        changed,
        "Voice candidate is no longer available for rejection",
    )
}

pub(crate) async fn select_candidate(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    candidate_id: &str,
    space_id: &str,
    profile_id: &str,
    revision_id: &str,
    now: i64,
) -> Result<(), String> {
    let changed = sqlx::query(
        "UPDATE spaces_voice_candidates SET status='selected',selected_voice_profile_id=?, \
         selected_voice_revision_id=?,selected_at=? WHERE id=? AND space_id=? \
         AND status='ready' AND auditioned_at IS NOT NULL",
    )
    .bind(profile_id)
    .bind(revision_id)
    .bind(now)
    .bind(candidate_id)
    .bind(space_id)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    db::require_one(
        changed,
        "Voice candidate must be auditioned before it can be selected",
    )
}

fn decode(row: sqlx::sqlite::SqliteRow) -> Result<StoredVoiceCandidate, String> {
    let status: String = row.try_get("status").map_err(db::sql_error)?;
    let status = match status.as_str() {
        "ready" => VoiceCandidateStatus::Ready,
        "selected" => VoiceCandidateStatus::Selected,
        "rejected" => VoiceCandidateStatus::Rejected,
        _ => return Err("Persisted voice candidate status is invalid".into()),
    };
    let sample_rate = u32::try_from(
        row.try_get::<i64, _>("preview_sample_rate")
            .map_err(db::sql_error)?,
    )
    .map_err(|_| "Persisted voice candidate sample rate is invalid")?;
    let sample_count = u64::try_from(
        row.try_get::<i64, _>("preview_sample_count")
            .map_err(db::sql_error)?,
    )
    .map_err(|_| "Persisted voice candidate sample count is invalid")?;
    let reference_byte_length = u64::try_from(
        row.try_get::<i64, _>("reference_byte_length")
            .map_err(db::sql_error)?,
    )
    .map_err(|_| "Persisted voice candidate reference length is invalid")?;
    Ok(StoredVoiceCandidate {
        reference_asset_path: row.try_get("reference_asset_path").map_err(db::sql_error)?,
        reference_sha256: row.try_get("reference_sha256").map_err(db::sql_error)?,
        reference_byte_length,
        candidate: VoiceCandidate {
            model: VoiceModel::from_specification(
                &row.try_get::<String, _>("model_spec_id")
                    .map_err(db::sql_error)?,
            )?,
            candidate_id: row.try_get("id").map_err(db::sql_error)?,
            run_id: row.try_get("run_id").map_err(db::sql_error)?,
            space_id: row.try_get("space_id").map_err(db::sql_error)?,
            source_kind: row.try_get("source_kind").map_err(db::sql_error)?,
            language: row.try_get("language").map_err(db::sql_error)?,
            sample_text: row.try_get("sample_text").map_err(db::sql_error)?,
            seed: u32::try_from(row.try_get::<i64, _>("seed").map_err(db::sql_error)?)
                .map_err(|_| "Voice candidate seed is invalid")?,
            design_description: row.try_get("design_description").map_err(db::sql_error)?,
            reference_text: row.try_get("reference_text").map_err(db::sql_error)?,
            audio: SpeechAudioOutput {
                path: row.try_get("preview_asset_path").map_err(db::sql_error)?,
                sha256: row.try_get("preview_sha256").map_err(db::sql_error)?,
                sample_rate,
                sample_count,
            },
            status,
            auditioned_at: row.try_get("auditioned_at").map_err(db::sql_error)?,
            selected_voice_profile_id: row
                .try_get("selected_voice_profile_id")
                .map_err(db::sql_error)?,
            selected_voice_revision_id: row
                .try_get("selected_voice_revision_id")
                .map_err(db::sql_error)?,
            selected_at: row.try_get("selected_at").map_err(db::sql_error)?,
            rejected_at: row.try_get("rejected_at").map_err(db::sql_error)?,
            created_at: row.try_get("created_at").map_err(db::sql_error)?,
        },
    })
}
