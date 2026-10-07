use crate::{app_error::AppError, spaces::db};

use super::{contracts::*, project_config::ProjectOutputConfig, repository, text};

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
    }?;
    match (
        segment.source_segment_id.as_deref(),
        segment.source_segment_revision,
        segment.source_text.as_deref(),
    ) {
        (None, None, None) => Ok(()),
        (Some(id), Some(revision), Some(text))
            if !id.trim().is_empty()
                && id.encode_utf16().count() <= 240
                && revision > 0
                && !text.trim().is_empty()
                && text.chars().count() <= 16_000 =>
        {
            Ok(())
        }
        _ => Err("Speech segment source identity is incomplete or invalid".into()),
    }
}

#[tauri::command]
pub async fn save_speech_project_revision(
    app: tauri::AppHandle,
    request: SaveSpeechProjectRevisionRequest,
) -> Result<SpeechProjectSnapshot, AppError> {
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
    let pronunciations = text::parse_lexicon(&request.pronunciation_lexicon)?;
    let lexicon = serde_json::to_string(&request.pronunciation_lexicon).map_err(db::json_error)?;
    let revision_id = uuid::Uuid::new_v4().to_string();
    let now = crate::clock::unix_time_millis()?;
    let pool = db::pool(&app).await?;
    let mut tx = db::begin_write(pool).await?;
    let (active_revision_id, active_revision, project_type, space_id, source_node_id, active_output): (
        String,
        i64,
        String,
        String,
        String,
        String,
    ) = sqlx::query_as(
        "SELECT r.id,r.revision,p.project_type,p.space_id,p.source_node_id,r.output_config FROM spaces_speech_projects p \
         JOIN spaces_speech_project_revisions r ON r.id=p.active_revision_id WHERE p.id=?",
    )
    .bind(&request.project_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db::sql_error)?
    .ok_or("Speech project no longer exists")?;
    if active_revision != request.expected_revision {
        return Err("stale_revision: Speech project changed before this revision was saved".into());
    }
    let config = super::project_config::parse(&project_type, &request.output_config)?;
    super::project_source::require(&mut tx, &space_id, &source_node_id, &config).await?;
    let output = serde_json::to_string(&config).map_err(db::json_error)?;
    if let ProjectOutputConfig::Audiobook(config) = &config {
        super::book_assets::require_cover(
            &mut tx,
            &space_id,
            Some(&request.project_id),
            config.metadata.cover.as_ref(),
        )
        .await?;
        super::project_config::validate_chapter_order(
            config,
            request
                .segments
                .iter()
                .map(|segment| segment.chapter_key.as_deref()),
        )?;
        super::project_config::validate_role_bindings(config, &request.segments)?;
        for role in &config.roles {
            repository::require_voice_in_space(
                &mut tx,
                &role.voice_revision_id,
                &request.project_id,
            )
            .await?;
        }
        if request
            .segments
            .iter()
            .any(|segment| segment.target_start_ms.is_some() || segment.source_segment_id.is_some())
        {
            return Err("Audiobook paragraphs must not contain video source metadata".into());
        }
    } else if let ProjectOutputConfig::VideoDubbing(config) = &config {
        super::video_config::require_sources(&mut tx, &space_id, Some(&request.project_id), config)
            .await?;
        for segment in &request.segments {
            super::video_config::validate_timing(
                config,
                segment.target_start_ms,
                segment.target_end_ms,
                segment.chapter_key.as_deref(),
            )?;
        }
        if config.translation.is_some()
            && request
                .segments
                .iter()
                .any(|segment| segment.source_segment_id.is_none())
        {
            return Err(
                "Translated video revisions require source identity for every segment".into(),
            );
        }
        let active_output: serde_json::Value =
            serde_json::from_str(&active_output).map_err(db::json_error)?;
        super::translation::validate_saved_provenance(
            &mut tx,
            super::translation::SaveContext {
                space_id: &space_id,
                project_id: &request.project_id,
                active_revision_id: &active_revision_id,
                active_revision,
                active_output_config: &active_output,
                requested_output_config: config,
                segments: &request.segments,
            },
        )
        .await?;
    } else if request.segments.iter().any(|segment| {
        segment.chapter_key.is_some()
            || segment.target_start_ms.is_some()
            || segment.source_segment_id.is_some()
    }) {
        return Err("Short dubbing must not contain chapter or video time constraints".into());
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
            let normalized = text::apply_lexicon(&text_segment, &pronunciations);
            if normalized.chars().count() > 10_000 {
                return Err(
                    "Pronunciation substitutions exceed the synthesis segment limit".into(),
                );
            }
            sqlx::query(
                "INSERT INTO spaces_speech_segments(id,project_revision_id,ordinal,chapter_key,speaker_key, \
                text,normalized_text,voice_revision_id,language,target_start_ms,target_end_ms, \
                 source_segment_id,source_segment_revision,source_text,state,created_at,updated_at) \
                 VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?,'ready',?,?)",
            )
            .bind(segment_id)
            .bind(&revision_id)
            .bind(ordinal)
            .bind(&segment.chapter_key)
            .bind(&segment.speaker_key)
            .bind(&text_segment)
            .bind(normalized)
            .bind(&segment.voice_revision_id)
            .bind(segment.language.trim())
            .bind(segment.target_start_ms)
            .bind(segment.target_end_ms)
            .bind(&segment.source_segment_id)
            .bind(segment.source_segment_revision.map(i64::from))
            .bind(segment.source_text.as_deref().map(str::trim))
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
