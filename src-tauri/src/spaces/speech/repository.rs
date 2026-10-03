use sqlx::{Sqlite, Transaction};

use super::contracts::{SpeechProjectDetail, SpeechProjectSummary, VoiceProfileSummary};
use crate::spaces::db;

pub async fn list_voices(
    pool: &sqlx::SqlitePool,
    space_id: &str,
) -> Result<Vec<VoiceProfileSummary>, String> {
    sqlx::query_as(
        "SELECT p.id, p.name, p.source_kind, p.active_revision_id, r.engine_id, r.model_spec_id, \
         r.language, r.preset_voice_id, p.updated_at FROM spaces_voice_profiles p \
         JOIN spaces_voice_revisions r ON r.id=p.active_revision_id WHERE p.space_id=? \
         ORDER BY p.updated_at DESC, p.id",
    )
    .bind(space_id)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)
}

pub async fn list_projects(
    pool: &sqlx::SqlitePool,
    space_id: &str,
) -> Result<Vec<SpeechProjectSummary>, String> {
    sqlx::query_as(
        "SELECT p.id, p.space_id, p.project_type, p.title, p.active_revision_id, r.revision AS active_revision, \
         COUNT(s.id) AS segment_count,p.generation_epoch,p.active_generation_job_id,p.updated_at \
         FROM spaces_speech_projects p \
         JOIN spaces_speech_project_revisions r ON r.id=p.active_revision_id \
         LEFT JOIN spaces_speech_segments s ON s.project_revision_id=r.id \
         WHERE p.space_id=? GROUP BY p.id ORDER BY p.updated_at DESC, p.id",
    )
    .bind(space_id)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)
}

pub async fn load_project(
    pool: &sqlx::SqlitePool,
    project_id: &str,
) -> Result<SpeechProjectDetail, String> {
    let project: SpeechProjectSummary = sqlx::query_as(
        "SELECT p.id, p.space_id, p.project_type, p.title, p.active_revision_id, r.revision AS active_revision, \
         COUNT(s.id) AS segment_count,p.generation_epoch,p.active_generation_job_id,p.updated_at \
         FROM spaces_speech_projects p \
         JOIN spaces_speech_project_revisions r ON r.id=p.active_revision_id \
         LEFT JOIN spaces_speech_segments s ON s.project_revision_id=r.id \
         WHERE p.id=? GROUP BY p.id",
    )
    .bind(project_id)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?
    .ok_or_else(|| "Speech project no longer exists".to_owned())?;
    let revision = sqlx::query_as(
        "SELECT id, project_id, revision, source_revision, pronunciation_lexicon, output_config, \
         output_asset_path,output_sha256,output_sample_rate,output_sample_count, \
         created_at FROM spaces_speech_project_revisions WHERE id=?",
    )
    .bind(&project.active_revision_id)
    .fetch_one(pool)
    .await
    .map_err(db::sql_error)?;
    let segments = sqlx::query_as(
        "SELECT id, ordinal, chapter_key, speaker_key, text, normalized_text, voice_revision_id, \
         language, target_start_ms, target_end_ms, state, state_reason \
         FROM spaces_speech_segments WHERE project_revision_id=? ORDER BY ordinal",
    )
    .bind(&project.active_revision_id)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    Ok(SpeechProjectDetail {
        project,
        revision,
        segments,
    })
}

pub async fn require_voice_in_space(
    tx: &mut Transaction<'_, Sqlite>,
    voice_revision_id: &str,
    project_id: &str,
) -> Result<String, String> {
    let language: Option<String> = sqlx::query_scalar(
        "SELECT r.language FROM spaces_voice_revisions r \
         JOIN spaces_voice_profiles v ON v.id=r.profile_id \
         JOIN spaces_speech_projects p ON p.space_id=v.space_id \
         WHERE r.id=? AND p.id=? AND v.source_kind='preset' \
           AND r.engine_id='kokoro' AND r.model_spec_id=?",
    )
    .bind(voice_revision_id)
    .bind(project_id)
    .bind(super::execution::KOKORO_MODEL_SPEC_ID)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    language
        .ok_or_else(|| "A compatible preset voice revision is required in this Space".to_owned())
}
