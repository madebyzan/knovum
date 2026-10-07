use sqlx::{Sqlite, Transaction};

use super::contracts::{
    SpeechProjectOverview, SpeechProjectRevision, SpeechProjectSnapshot, SpeechSegment,
    VoiceLibraryRevision,
};
use crate::spaces::db;

pub async fn list_voices(
    pool: &sqlx::SqlitePool,
    space_id: &str,
) -> Result<Vec<VoiceLibraryRevision>, String> {
    sqlx::query_as(
        "SELECT p.id AS profile_id,p.name,p.source_kind,r.id AS revision_id,r.revision, \
         r.id=p.active_revision_id AS is_active,r.engine_id,r.model_spec_id,r.language,r.preset_voice_id, \
         p.archived_at,p.updated_at FROM spaces_voice_profiles p JOIN spaces_voice_revisions r ON r.profile_id=p.id \
         WHERE p.space_id=? ORDER BY p.archived_at IS NOT NULL,p.updated_at DESC,r.id=p.active_revision_id DESC,r.revision DESC",
    )
    .bind(space_id)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)
}

pub async fn list_projects(
    pool: &sqlx::SqlitePool,
    space_id: &str,
    source_node_id: &str,
) -> Result<Vec<SpeechProjectOverview>, String> {
    sqlx::query_as(
        "SELECT p.id, p.space_id, p.source_node_id, p.project_type, p.title, p.active_revision_id, r.revision AS active_revision, \
         COUNT(s.id) AS segment_count,(SELECT r0.id FROM spaces_runs r0 \
           WHERE r0.kind='workflow' AND r0.completed_at IS NULL \
             AND json_extract(r0.input_snapshot,'$.workflow')='speech_project_generation_v1' \
             AND json_extract(r0.input_snapshot,'$.projectId')=p.id \
           ORDER BY r0.created_at DESC LIMIT 1) AS active_run_id,p.updated_at \
         FROM spaces_speech_projects p \
         JOIN spaces_speech_project_revisions r ON r.id=p.active_revision_id \
         LEFT JOIN spaces_speech_segments s ON s.project_revision_id=r.id \
         WHERE p.space_id=? AND p.source_node_id=? GROUP BY p.id ORDER BY p.updated_at DESC, p.id",
    )
    .bind(space_id)
    .bind(source_node_id)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)
}

pub async fn load_project(
    pool: &sqlx::SqlitePool,
    project_id: &str,
) -> Result<SpeechProjectSnapshot, String> {
    let project: SpeechProjectOverview = sqlx::query_as(
        "SELECT p.id, p.space_id, p.source_node_id, p.project_type, p.title, p.active_revision_id, r.revision AS active_revision, \
         COUNT(s.id) AS segment_count,(SELECT r0.id FROM spaces_runs r0 \
           WHERE r0.kind='workflow' AND r0.completed_at IS NULL \
             AND json_extract(r0.input_snapshot,'$.workflow')='speech_project_generation_v1' \
             AND json_extract(r0.input_snapshot,'$.projectId')=p.id \
           ORDER BY r0.created_at DESC LIMIT 1) AS active_run_id,p.updated_at \
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
    let revision: SpeechProjectRevision = sqlx::query_as(
        "SELECT id, project_id, revision, source_revision, pronunciation_lexicon, output_config, \
         output_asset_path,output_sha256,output_sample_rate,output_sample_count, \
         created_at FROM spaces_speech_project_revisions WHERE id=?",
    )
    .bind(&project.active_revision_id)
    .fetch_one(pool)
    .await
    .map_err(db::sql_error)?;
    revision.output_config.validate(&project.project_type)?;
    super::text::validate_lexicon(&revision.pronunciation_lexicon)?;
    let segments = load_segments(pool, &project.active_revision_id).await?;
    Ok(SpeechProjectSnapshot {
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
         WHERE r.id=? AND p.id=? AND ( \
           (v.source_kind='preset' AND r.engine_id='kokoro' AND r.model_spec_id=?) OR \
           (v.source_kind IN ('reference','designed') AND r.engine_id='voxcpm2' \
             AND r.model_spec_id=? \
             AND r.reference_asset_id IS NOT NULL AND r.reference_source_revision IS NOT NULL \
             AND (v.source_kind='reference' OR \
               (r.reference_text IS NOT NULL AND length(trim(r.reference_text)) > 0))))",
    )
    .bind(voice_revision_id)
    .bind(project_id)
    .bind(super::execution::KOKORO_MODEL_SPEC_ID)
    .bind(super::voice_candidates::MODEL_SPEC_ID)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    language.ok_or_else(|| "A compatible voice revision is required in this Space".to_owned())
}

pub(super) async fn load_segments(
    pool: &sqlx::SqlitePool,
    revision_id: &str,
) -> Result<Vec<SpeechSegment>, String> {
    sqlx::query_as(
        "SELECT id, ordinal, chapter_key, speaker_key, text, normalized_text, voice_revision_id, \
         language, target_start_ms, target_end_ms,source_segment_id,source_segment_revision, \
         source_text,state, state_reason \
         FROM spaces_speech_segments WHERE project_revision_id=? ORDER BY ordinal",
    )
    .bind(revision_id)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)
}
