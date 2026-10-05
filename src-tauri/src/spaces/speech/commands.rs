use crate::{app_error::AppError, spaces::db};

use super::{contracts::*, repository};

fn required(value: &str, field: &str, max: usize) -> Result<(), String> {
    let length = value.trim().chars().count();
    if length == 0 || length > max {
        return Err(format!(
            "{field} must contain between 1 and {max} characters"
        ));
    }
    Ok(())
}

fn object_json(value: serde_json::Value, field: &str) -> Result<String, String> {
    if !value.is_object() {
        return Err(format!("{field} must be a JSON object"));
    }
    serde_json::to_string(&value).map_err(db::json_error)
}

#[tauri::command]
pub async fn list_voice_revisions(
    app: tauri::AppHandle,
    space_id: String,
) -> Result<Vec<VoiceLibraryRevision>, AppError> {
    Ok(repository::list_voices(db::pool(&app).await?, &space_id).await?)
}

#[tauri::command]
pub async fn create_preset_voice(
    app: tauri::AppHandle,
    request: CreatePresetVoiceRequest,
) -> Result<VoiceLibraryRevision, AppError> {
    required(&request.name, "Voice name", 120)?;
    required(&request.preset_voice_id, "Preset voice", 120)?;
    required(&request.engine_id, "Engine identity", 160)?;
    required(&request.model_spec_id, "Model specification", 240)?;
    required(&request.language, "Language", 32)?;
    if request.engine_id != "kokoro" {
        return Err("Preset voices require the Kokoro synthesis engine".into());
    }
    if request.model_spec_id != super::execution::KOKORO_MODEL_SPEC_ID {
        return Err("Preset voice model specification is not supported".into());
    }
    super::preset_voices::require(&request.preset_voice_id, &request.language)?;
    let parsed_config = parse_synthesis_config(&request.synthesis_config)?;
    if parsed_config.sample_rate != 24_000 {
        return Err("Kokoro voices require 24 kHz synthesis".into());
    }
    let config = serde_json::to_string(&parsed_config).map_err(db::json_error)?;
    let profile_id = uuid::Uuid::new_v4().to_string();
    let revision_id = uuid::Uuid::new_v4().to_string();
    let now = crate::clock::unix_time_millis()?;
    let pool = db::pool(&app).await?;
    let mut tx = db::begin_write(pool).await?;
    sqlx::query(
        "INSERT INTO spaces_voice_profiles(id,space_id,name,source_kind,active_revision_id,created_at,updated_at) \
         VALUES(?,?,?,'preset',NULL,?,?)",
    )
    .bind(&profile_id)
    .bind(&request.space_id)
    .bind(request.name.trim())
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    sqlx::query(
        "INSERT INTO spaces_voice_revisions(id,profile_id,revision,engine_id,model_spec_id,language, \
         preset_voice_id,synthesis_config,authorization,created_at) VALUES(?,?,1,?,?,?,?,?,'{}',?)",
    )
    .bind(&revision_id)
    .bind(&profile_id)
    .bind(request.engine_id.trim())
    .bind(request.model_spec_id.trim())
    .bind(request.language.trim())
    .bind(request.preset_voice_id.trim())
    .bind(config)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    sqlx::query("UPDATE spaces_voice_profiles SET active_revision_id=? WHERE id=?")
        .bind(&revision_id)
        .bind(&profile_id)
        .execute(&mut *tx)
        .await
        .map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(repository::list_voices(pool, &request.space_id)
        .await?
        .into_iter()
        .find(|profile| profile.profile_id == profile_id)
        .ok_or("Created voice profile could not be loaded")?)
}

#[tauri::command]
pub async fn list_speech_projects(
    app: tauri::AppHandle,
    space_id: String,
) -> Result<Vec<SpeechProjectSummary>, AppError> {
    Ok(repository::list_projects(db::pool(&app).await?, &space_id).await?)
}

#[tauri::command]
pub async fn create_speech_project(
    app: tauri::AppHandle,
    request: CreateSpeechProjectRequest,
) -> Result<SpeechProjectSummary, AppError> {
    required(&request.title, "Project title", 200)?;
    super::project_config::validate(&request.project_type, &request.output_config)?;
    if request.project_type == "audiobook" {
        let config = super::project_config::parse_audiobook(&request.output_config)?;
        if !config.chapters.is_empty() || !config.roles.is_empty() {
            return Err("A new audiobook project must start with an empty script".into());
        }
    }
    let output = object_json(request.output_config.clone(), "Output configuration")?;
    let project_id = uuid::Uuid::new_v4().to_string();
    let revision_id = uuid::Uuid::new_v4().to_string();
    let source_revision = uuid::Uuid::new_v4().to_string();
    let now = crate::clock::unix_time_millis()?;
    let pool = db::pool(&app).await?;
    let mut tx = db::begin_write(pool).await?;
    if request.project_type == "audiobook" {
        let config = super::project_config::parse_audiobook(&request.output_config)?;
        super::book_assets::require_cover(
            &mut tx,
            &request.space_id,
            None,
            config.metadata.cover.as_ref(),
        )
        .await?;
    }
    if request.project_type == "video_dubbing" {
        let config = super::video_config::parse(&request.output_config)?;
        super::video_config::require_sources(&mut tx, &request.space_id, None, &config).await?;
    }
    sqlx::query(
        "INSERT INTO spaces_speech_projects(id,space_id,project_type,title,active_revision_id,created_at,updated_at) \
         VALUES(?,?,?,?,NULL,?,?)",
    )
    .bind(&project_id)
    .bind(&request.space_id)
    .bind(&request.project_type)
    .bind(request.title.trim())
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    sqlx::query(
        "INSERT INTO spaces_speech_project_revisions(id,project_id,revision,source_revision, \
         pronunciation_lexicon,output_config,created_at) VALUES(?,?,1,?,'{}',?,?)",
    )
    .bind(&revision_id)
    .bind(&project_id)
    .bind(source_revision)
    .bind(output)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    sqlx::query("UPDATE spaces_speech_projects SET active_revision_id=? WHERE id=?")
        .bind(&revision_id)
        .bind(&project_id)
        .execute(&mut *tx)
        .await
        .map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(repository::list_projects(pool, &request.space_id)
        .await?
        .into_iter()
        .find(|project| project.id == project_id)
        .ok_or("Created speech project could not be loaded")?)
}

#[tauri::command]
pub async fn get_speech_project(
    app: tauri::AppHandle,
    project_id: String,
) -> Result<SpeechProjectDetail, AppError> {
    Ok(repository::load_project(db::pool(&app).await?, &project_id).await?)
}
