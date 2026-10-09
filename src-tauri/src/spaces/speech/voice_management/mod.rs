mod archive;
mod contracts;
mod delete;
mod repository;

pub use contracts::{
    ArchiveVoiceProfileRequest, DeleteVoiceProfileRequest, RenameVoiceProfileRequest,
    ReviseVoiceProfileRequest, UnarchiveVoiceProfileRequest, VoiceProfileHistory, VoiceRevision,
};

use crate::{app_error::AppError, spaces::db};

pub(super) fn validate_name(name: &str) -> Result<(), String> {
    if !(1..=120).contains(&name.trim().chars().count()) {
        return Err("Voice name must contain between 1 and 120 characters".into());
    }
    Ok(())
}

#[tauri::command]
pub async fn list_voice_profiles(
    app: tauri::AppHandle,
) -> Result<Vec<VoiceProfileHistory>, AppError> {
    Ok(repository::list(db::pool(&app).await?).await?)
}

#[tauri::command]
pub async fn rename_voice_profile(
    app: tauri::AppHandle,
    request: RenameVoiceProfileRequest,
) -> Result<(), AppError> {
    validate_name(&request.name)?;
    let changed = sqlx::query(
        "UPDATE spaces_voice_profiles SET name=?,updated_at=? WHERE id=? AND updated_at=?",
    )
    .bind(request.name.trim())
    .bind(crate::clock::unix_time_millis()?)
    .bind(&request.profile_id)
    .bind(request.expected_updated_at)
    .execute(db::pool(&app).await?)
    .await
    .map_err(db::sql_error)?;
    db::require_one(changed, "Voice profile changed before it could be renamed")?;
    Ok(())
}

#[tauri::command]
pub async fn revise_voice_profile(
    app: tauri::AppHandle,
    request: ReviseVoiceProfileRequest,
) -> Result<(), AppError> {
    if !matches!(request.language.as_str(), "zh-CN" | "en") {
        return Err("Voice language is unsupported".into());
    }
    let pool = db::pool(&app).await?;
    let mut tx = db::begin_write(pool).await?;
    let current: RevisionSource = sqlx::query_as(
        "SELECT r.revision,r.engine_id,r.model_spec_id,r.preset_voice_id,r.reference_asset_id, \
         r.reference_source_revision,r.reference_text,r.design_description,r.authorization,r.synthesis_config \
         FROM spaces_voice_profiles p JOIN spaces_voice_revisions r ON r.id=p.active_revision_id \
         WHERE p.id=? AND p.active_revision_id=? AND p.archived_at IS NULL",
    )
    .bind(&request.profile_id)
    .bind(&request.expected_active_revision_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db::sql_error)?
    .ok_or("Voice profile changed or is archived")?;
    let previous = super::contracts::parse_synthesis_config(
        &serde_json::from_str(&current.synthesis_config).map_err(db::json_error)?,
    )?;
    if let Some(speaker) = &current.preset_voice_id {
        super::preset_voices::require(speaker, &request.language)?;
    }
    let config = revision_config(&current.engine_id, request.speed, previous.seed)?;
    let revision_id = uuid::Uuid::new_v4().to_string();
    let now = crate::clock::unix_time_millis()?;
    sqlx::query("INSERT INTO spaces_voice_revisions(id,profile_id,revision,engine_id,model_spec_id,language,preset_voice_id,reference_asset_id,reference_source_revision,reference_text,design_description,synthesis_config,authorization,created_at) VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?)")
        .bind(&revision_id).bind(&request.profile_id).bind(current.revision + 1).bind(&current.engine_id)
        .bind(&current.model_spec_id).bind(&request.language).bind(&current.preset_voice_id)
        .bind(&current.reference_asset_id).bind(&current.reference_source_revision).bind(&current.reference_text)
        .bind(&current.design_description).bind(sqlx::types::Json(&config)).bind(&current.authorization).bind(now)
        .execute(&mut *tx).await.map_err(db::sql_error)?;
    let changed = sqlx::query("UPDATE spaces_voice_profiles SET active_revision_id=?,updated_at=? WHERE id=? AND active_revision_id=? AND archived_at IS NULL")
        .bind(&revision_id).bind(now).bind(&request.profile_id).bind(&request.expected_active_revision_id)
        .execute(&mut *tx).await.map_err(db::sql_error)?;
    db::require_one(
        changed,
        "Voice profile changed before the revision was published",
    )?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(())
}

#[tauri::command]
pub async fn archive_voice_profile(
    app: tauri::AppHandle,
    request: ArchiveVoiceProfileRequest,
) -> Result<(), AppError> {
    archive::update_state(
        db::pool(&app).await?,
        &request.profile_id,
        request.expected_updated_at,
        true,
    )
    .await?;
    Ok(())
}

#[tauri::command]
pub async fn unarchive_voice_profile(
    app: tauri::AppHandle,
    request: UnarchiveVoiceProfileRequest,
) -> Result<(), AppError> {
    archive::update_state(
        db::pool(&app).await?,
        &request.profile_id,
        request.expected_updated_at,
        false,
    )
    .await?;
    Ok(())
}

#[tauri::command]
pub async fn delete_voice_profile(
    app: tauri::AppHandle,
    request: DeleteVoiceProfileRequest,
) -> Result<(), AppError> {
    delete::execute(&app, &request).await?;
    Ok(())
}

#[derive(sqlx::FromRow)]
struct RevisionSource {
    revision: i64,
    engine_id: String,
    model_spec_id: String,
    preset_voice_id: Option<String>,
    reference_asset_id: Option<String>,
    reference_source_revision: Option<String>,
    reference_text: Option<String>,
    design_description: Option<String>,
    authorization: String,
    synthesis_config: String,
}

fn revision_config(
    engine_id: &str,
    speed: f32,
    seed: u32,
) -> Result<super::contracts::VoiceSynthesisConfig, String> {
    let sample_rate = match engine_id {
        "moss_nano" if speed == 1.0 => 48_000,
        "voxcpm2" if speed == 1.0 => 48_000,
        "moss_nano" => return Err("MOSS-TTS-Nano requires normal speed".into()),
        "voxcpm2" => return Err("VoxCPM2 uses its fixed speed profile".into()),
        _ => return Err("Voice engine is unsupported".into()),
    };
    Ok(super::contracts::VoiceSynthesisConfig {
        speed,
        sample_rate,
        seed,
    })
}
