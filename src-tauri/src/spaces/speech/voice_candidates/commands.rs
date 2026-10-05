use super::{
    repository::{list_ready, load_candidate, load_record, mark_auditioned, reject_candidate},
    DraftVoiceCandidateRequest, VoiceCandidate, WORKFLOW,
};
use crate::{app_error::AppError, spaces::db, spaces::runs::RunObservation};

#[tauri::command]
pub async fn draft_voice_candidate(
    app: tauri::AppHandle,
    request: DraftVoiceCandidateRequest,
) -> Result<RunObservation, AppError> {
    Ok(super::start::start(&app, request).await?)
}

#[tauri::command]
pub async fn get_voice_candidate(
    app: tauri::AppHandle,
    candidate_id: String,
    space_id: String,
) -> Result<VoiceCandidate, AppError> {
    super::require_uuid(&candidate_id, "Voice candidate identity")?;
    Ok(load_candidate(db::pool(&app).await?, &candidate_id, &space_id).await?)
}

#[tauri::command]
pub async fn list_voice_candidates(
    app: tauri::AppHandle,
    space_id: String,
) -> Result<Vec<VoiceCandidate>, AppError> {
    Ok(list_ready(db::pool(&app).await?, &space_id).await?)
}

#[tauri::command]
pub async fn get_active_voice_candidate_run(
    app: tauri::AppHandle,
    space_id: String,
) -> Result<Option<RunObservation>, AppError> {
    let pool = db::pool(&app).await?;
    let run_id: Option<String> = sqlx::query_scalar(
        "SELECT id FROM spaces_runs WHERE home_space_id=? AND completed_at IS NULL \
         AND json_extract(input_snapshot,'$.workflow')=? ORDER BY created_at DESC,id DESC LIMIT 1",
    )
    .bind(&space_id)
    .bind(WORKFLOW)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?;
    match run_id {
        Some(run_id) => Ok(Some(crate::spaces::runs::load_run(pool, &run_id).await?)),
        None => Ok(None),
    }
}

#[tauri::command]
pub async fn mark_voice_candidate_auditioned(
    app: tauri::AppHandle,
    candidate_id: String,
    space_id: String,
) -> Result<VoiceCandidate, AppError> {
    super::require_uuid(&candidate_id, "Voice candidate identity")?;
    let pool = db::pool(&app).await?;
    let record = load_record(pool, &candidate_id, &space_id).await?;
    let path = crate::spaces::assets::authorize_spaces_asset(
        &app,
        std::path::Path::new(&record.candidate.audio.path),
    )?;
    let expected = record.candidate.audio.sha256;
    let actual = crate::blocking::run(move || super::super::publication::hash_file(&path)).await?;
    if actual != expected {
        return Err("stale_revision: voice candidate changed before audition review".into());
    }
    mark_auditioned(
        pool,
        &candidate_id,
        &space_id,
        crate::clock::unix_time_millis()?,
    )
    .await?;
    Ok(load_candidate(pool, &candidate_id, &space_id).await?)
}

#[tauri::command]
pub async fn discard_voice_candidate(
    app: tauri::AppHandle,
    candidate_id: String,
    space_id: String,
) -> Result<(), AppError> {
    super::require_uuid(&candidate_id, "Voice candidate identity")?;
    reject_candidate(db::pool(&app).await?, &candidate_id, &space_id).await?;
    Ok(())
}
