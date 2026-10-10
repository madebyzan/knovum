use crate::{ai::contracts::speech_wire::AudioFileReference, app_error::AppError, spaces::db};

use super::{
    contracts::{CreateCandidateVoiceRequest, VoiceLibraryRevision},
    voice_assets,
    voice_candidates::{self, VoiceCandidateStatus},
};

#[tauri::command]
pub async fn create_candidate_voice(
    app: tauri::AppHandle,
    request: CreateCandidateVoiceRequest,
) -> Result<VoiceLibraryRevision, AppError> {
    if !(1..=120).contains(&request.name.trim().chars().count()) {
        return Err("Voice name must contain between 1 and 120 characters".into());
    }
    let pool = db::pool(&app).await?;
    let record = voice_candidates::load_record(pool, &request.candidate_id).await?;
    if record.candidate.status == VoiceCandidateStatus::Selected {
        return load_selected(pool, &request).await.map_err(Into::into);
    }
    if record.candidate.auditioned_at.is_none() {
        return Err("Voice candidate must be auditioned before it can be selected".into());
    }
    let preview = crate::spaces::assets::authorize_spaces_asset(
        &app,
        std::path::Path::new(&record.candidate.audio.path),
    )?;
    let preview_sha256 = record.candidate.audio.sha256.clone();
    let actual_preview =
        crate::blocking::run(move || super::publication::hash_file(&preview)).await?;
    if actual_preview != preview_sha256 {
        return Err("stale_revision: voice candidate changed after audition".into());
    }
    let reference = AudioFileReference {
        path: record.reference_asset_path,
        media_type: "audio/wav".into(),
        byte_length: record.reference_byte_length,
        sha256: record.reference_sha256,
        sample_rate: 16_000,
        channels: 1,
        sample_format: "pcm_s16le".into(),
    };
    let persisted =
        voice_assets::persist_reference(&app, &reference, &request.candidate_id).await?;
    let revision_id = uuid::Uuid::new_v4().to_string();
    let now = crate::clock::unix_time_millis()?;
    let creation = async {
        let mut tx = db::begin_write(pool).await?;
        sqlx::query("INSERT INTO spaces_voice_profiles(id,name,source_kind,active_revision_id,created_at,updated_at) VALUES(?,?,?,NULL,?,?)")
            .bind(&request.candidate_id).bind(request.name.trim())
            .bind(&record.candidate.source_kind).bind(now).bind(now)
            .execute(&mut *tx).await.map_err(db::sql_error)?;
        let authorization = serde_json::json!({
            "referenceTranscriptConfirmed": record.candidate.reference_text.is_some(),
            "candidateId": request.candidate_id,
            "candidateRunId": record.candidate.run_id,
        });
        sqlx::query("INSERT INTO spaces_voice_revisions(id,profile_id,revision,engine_id,model_spec_id,language,preset_voice_id,reference_asset_id,reference_source_revision,reference_text,design_description,synthesis_config,authorization,created_at) VALUES(?,?,1,?,?,?,NULL,?,?,?,?,?,?,?)")
            .bind(&revision_id).bind(&request.candidate_id).bind(record.candidate.model.engine()).bind(record.candidate.model.specification())
            .bind(&record.candidate.language).bind(persisted.path.to_string_lossy().as_ref())
            .bind(format!("sha256:{}", reference.sha256)).bind(&record.candidate.reference_text)
            .bind(&record.candidate.design_description).bind(sqlx::types::Json(serde_json::json!({"speed":1.0,"sampleRate":48000,"seed":record.candidate.seed})))
            .bind(sqlx::types::Json(&authorization)).bind(now)
            .execute(&mut *tx).await.map_err(db::sql_error)?;
        sqlx::query("UPDATE spaces_voice_profiles SET active_revision_id=? WHERE id=?")
            .bind(&revision_id).bind(&request.candidate_id)
            .execute(&mut *tx).await.map_err(db::sql_error)?;
        voice_candidates::select_candidate(
            &mut tx,
            &request.candidate_id,
            &request.candidate_id,
            &revision_id,
            now,
        ).await?;
        super::asset_publication::remove_in_transaction(&mut tx, &persisted.publication_id).await?;
        tx.commit().await.map_err(db::sql_error)
    }.await;
    if let Err(error) = creation {
        let selected: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM spaces_voice_candidates WHERE id=? AND status='selected')",
        )
        .bind(&request.candidate_id)
        .fetch_one(pool)
        .await
        .map_err(db::sql_error)?;
        if !selected {
            super::asset_publication::abandon(&app, pool, &persisted.publication_id)
                .await
                .map_err(|cleanup| format!("{error}; voice reference cleanup failed: {cleanup}"))?;
            return Err(error.into());
        }
    }
    load_selected(pool, &request).await.map_err(Into::into)
}

async fn load_selected(
    pool: &sqlx::SqlitePool,
    request: &CreateCandidateVoiceRequest,
) -> Result<VoiceLibraryRevision, String> {
    sqlx::query_as::<_, VoiceLibraryRevision>(
        "SELECT p.id AS profile_id,p.name,p.source_kind,r.id AS revision_id,r.revision,(r.id=p.active_revision_id) AS is_active,r.engine_id,r.model_spec_id, \
         r.language,r.preset_voice_id, \
         (r.engine_id='voxcpm2' AND p.source_kind='reference' AND r.reference_text IS NULL) AS supports_delivery_instruction, \
         p.archived_at,p.updated_at FROM spaces_voice_profiles p \
         JOIN spaces_voice_candidates c ON c.selected_voice_profile_id=p.id \
         JOIN spaces_voice_revisions r ON r.id=c.selected_voice_revision_id AND r.profile_id=p.id \
         WHERE c.id=? AND c.status='selected'",
    )
    .bind(&request.candidate_id)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?
    .ok_or("Selected voice candidate has no matching voice revision".into())
}
