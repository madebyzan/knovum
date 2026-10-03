use crate::{app_error::AppError, spaces::db};

use super::{contracts::*, repository, text};

fn serialize_object(value: serde_json::Value, field: &str) -> Result<String, String> {
    if !value.is_object() {
        return Err(format!("{field} must be a JSON object"));
    }
    serde_json::to_string(&value).map_err(db::json_error)
}

fn validate_segment(segment: &SpeechSegmentInput) -> Result<(), String> {
    if segment.id.trim().is_empty() {
        return Err("Speech segment identity must not be empty".to_owned());
    }
    uuid::Uuid::parse_str(&segment.id).map_err(|_| "Speech segment identity must be a UUID")?;
    if segment.text.trim().is_empty() {
        return Err("Speech segment text must not be empty".to_owned());
    }
    if segment.language.trim().is_empty() {
        return Err("Speech segment language must not be empty".to_owned());
    }
    match (segment.target_start_ms, segment.target_end_ms) {
        (Some(start), Some(end)) if start >= 0 && end > start => Ok(()),
        (None, None) => Ok(()),
        _ => Err("Speech segment timing must contain a valid start and end pair".to_owned()),
    }
}

#[tauri::command]
pub async fn save_speech_project_revision(
    app: tauri::AppHandle,
    request: SaveSpeechProjectRevisionRequest,
) -> Result<SpeechProjectDetail, AppError> {
    if request.source_revision.trim().is_empty() {
        return Err("Source revision must not be empty".into());
    }
    if request.segments.is_empty() || request.segments.len() > 10_000 {
        return Err("A speech revision must contain between 1 and 10000 source segments".into());
    }
    let script_characters = request
        .segments
        .iter()
        .try_fold(0_usize, |total, segment| {
            total
                .checked_add(segment.text.chars().count())
                .ok_or("Speech script is too large")
        })?;
    if script_characters > 1_000_000 {
        return Err("Speech script exceeds one million characters".into());
    }
    for segment in &request.segments {
        validate_segment(segment)?;
    }
    if request
        .output_config
        .get("format")
        .and_then(serde_json::Value::as_str)
        != Some("wav")
    {
        return Err("Short dubbing revisions require WAV output".into());
    }
    let lexicon = serialize_object(request.pronunciation_lexicon, "Pronunciation lexicon")?;
    let output = serialize_object(request.output_config, "Output configuration")?;
    let revision_id = uuid::Uuid::new_v4().to_string();
    let now = crate::clock::unix_time_millis()?;
    let pool = db::pool(&app).await?;
    let mut tx = db::begin_write(pool).await?;
    let (active_revision, project_type): (i64, String) = sqlx::query_as(
        "SELECT r.revision,p.project_type FROM spaces_speech_projects p \
         JOIN spaces_speech_project_revisions r ON r.id=p.active_revision_id WHERE p.id=?",
    )
    .bind(&request.project_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db::sql_error)?
    .ok_or("Speech project no longer exists")?;
    if project_type != "short_dubbing" {
        return Err("Only short dubbing projects can be edited in this workflow".into());
    }
    if active_revision != request.expected_revision {
        return Err("stale_revision: Speech project changed before this revision was saved".into());
    }
    let next_revision = active_revision + 1;
    sqlx::query(
        "INSERT INTO spaces_speech_project_revisions(id,project_id,revision,source_revision, \
         pronunciation_lexicon,output_config,created_at) VALUES(?,?,?,?,?,?,?)",
    )
    .bind(&revision_id)
    .bind(&request.project_id)
    .bind(next_revision)
    .bind(request.source_revision.trim())
    .bind(lexicon)
    .bind(output)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    let mut ordinal = 0_i64;
    for segment in &request.segments {
        let voice_language = repository::require_voice_in_space(
            &mut tx,
            &segment.voice_revision_id,
            &request.project_id,
        )
        .await?;
        if voice_language != segment.language.trim() {
            return Err("Speech segment language differs from its voice revision".into());
        }
        let text_segments = text::segment(&segment.text)?;
        if text_segments.len() > 1 && segment.target_start_ms.is_some() {
            return Err("Timed speech segments must already contain one semantic segment".into());
        }
        let preserve_id = text_segments.len() == 1;
        for text_segment in text_segments {
            let segment_id = if preserve_id {
                segment.id.clone()
            } else {
                uuid::Uuid::new_v4().to_string()
            };
            sqlx::query(
                "INSERT INTO spaces_speech_segments(id,project_revision_id,ordinal,chapter_key,speaker_key, \
                 text,normalized_text,voice_revision_id,language,target_start_ms,target_end_ms,state, \
                 created_at,updated_at) VALUES(?,?,?,?,?,?,?,?,?,?,?,'ready',?,?)",
            )
            .bind(segment_id)
            .bind(&revision_id)
            .bind(ordinal)
            .bind(&segment.chapter_key)
            .bind(&segment.speaker_key)
            .bind(&text_segment)
            .bind(text::normalize(&text_segment))
            .bind(&segment.voice_revision_id)
            .bind(segment.language.trim())
            .bind(segment.target_start_ms)
            .bind(segment.target_end_ms)
            .bind(now)
            .bind(now)
            .execute(&mut *tx)
            .await
            .map_err(db::sql_error)?;
            ordinal = ordinal.checked_add(1).ok_or("Too many speech segments")?;
        }
    }
    let updated = sqlx::query(
        "UPDATE spaces_speech_projects SET active_revision_id=?,updated_at=? \
         WHERE id=? AND active_revision_id=(SELECT id FROM spaces_speech_project_revisions \
         WHERE project_id=? AND revision=?)",
    )
    .bind(&revision_id)
    .bind(now)
    .bind(&request.project_id)
    .bind(&request.project_id)
    .bind(request.expected_revision)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    db::require_one(
        updated,
        "stale_revision: Speech project changed before publication",
    )?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(repository::load_project(pool, &request.project_id).await?)
}
