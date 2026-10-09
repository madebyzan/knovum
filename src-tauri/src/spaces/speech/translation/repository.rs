use std::collections::HashSet;

use sqlx::{Row, Sqlite, SqlitePool, Transaction};

use super::{
    contracts::{
        TranslateVideoDubbingRequest, TranslationRunInput, TranslationStepConfiguration,
        VideoTranslation, VideoTranslationSegmentOutput,
    },
    WORKFLOW,
};
use crate::spaces::{authorization::RunScope, db};

pub(super) struct ActiveProject {
    pub space_id: String,
    pub project_type: String,
    pub revision_id: String,
    pub revision: i64,
    pub translated: bool,
}

pub(super) fn validate_request(request: &TranslateVideoDubbingRequest) -> Result<(), String> {
    require_canonical_uuid(&request.job_id, "Video translation request identity")?;
    require_canonical_uuid(&request.project_id, "Speech project identity")?;
    crate::ai::capabilities::translation::validate_target(&request.target_language)
        .map_err(|error| error.to_string())?;
    if request.expected_revision < 1
        || request.segments.is_empty()
        || request.segments.len() > 10_000
    {
        return Err(
            "Video translation requires a current project revision and 1 to 10000 segments".into(),
        );
    }
    let mut identities = HashSet::with_capacity(request.segments.len());
    let mut characters = 0_usize;
    for segment in &request.segments {
        let id = segment.source_segment_id.trim();
        let text = segment.text.trim();
        if id.is_empty()
            || id.encode_utf16().count() > 240
            || segment.source_segment_revision == 0
            || text.is_empty()
            || text.chars().count() > 10_000
            || !identities.insert(id)
        {
            return Err("Video translation segment identity or text is invalid".into());
        }
        characters = characters
            .checked_add(text.chars().count())
            .ok_or("Video translation input is too large")?;
    }
    if characters > 1_000_000 {
        return Err("Video translation input exceeds one million characters".into());
    }
    Ok(())
}

pub(super) async fn active_project(
    tx: &mut Transaction<'_, Sqlite>,
    project_id: &str,
) -> Result<ActiveProject, String> {
    let row = sqlx::query(
        "SELECT p.space_id,p.project_type,p.active_revision_id,r.revision, \
         json_type(r.output_config,'$.translation') AS translation_type \
         FROM spaces_speech_projects p JOIN spaces_speech_project_revisions r \
         ON r.id=p.active_revision_id WHERE p.id=?",
    )
    .bind(project_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)?
    .ok_or("Speech project no longer exists")?;
    Ok(ActiveProject {
        space_id: row.try_get("space_id").map_err(db::sql_error)?,
        project_type: row.try_get("project_type").map_err(db::sql_error)?,
        revision_id: row.try_get("active_revision_id").map_err(db::sql_error)?,
        revision: row.try_get("revision").map_err(db::sql_error)?,
        translated: row
            .try_get::<Option<String>, _>("translation_type")
            .map_err(db::sql_error)?
            .as_deref()
            == Some("object"),
    })
}

pub(super) fn require_expected_revision(
    project: &ActiveProject,
    expected_revision: i64,
) -> Result<(), String> {
    if project.project_type != "video_dubbing" {
        return Err("Translation is available only for video dubbing projects".into());
    }
    if project.revision != expected_revision {
        return Err("stale_revision: speech project changed before translation".into());
    }
    Ok(())
}

pub(super) async fn require_current_segments(
    tx: &mut Transaction<'_, Sqlite>,
    revision_id: &str,
    translated: bool,
    requested: &[super::contracts::VideoTranslationSegmentInput],
) -> Result<(), String> {
    if current_segments_match(tx, revision_id, translated, requested).await? {
        return Ok(());
    }
    Err("stale_revision: video translation input differs from the active script".into())
}

pub(super) async fn require_run_source_segments(
    tx: &mut Transaction<'_, Sqlite>,
    revision_id: &str,
    translated: bool,
    requested: &[super::contracts::VideoTranslationSegmentInput],
) -> Result<(), String> {
    if current_segments_match(tx, revision_id, translated, requested).await? {
        return Ok(());
    }
    Err("Video translation Run source snapshot differs from its base revision".into())
}

async fn current_segments_match(
    tx: &mut Transaction<'_, Sqlite>,
    revision_id: &str,
    translated: bool,
    requested: &[super::contracts::VideoTranslationSegmentInput],
) -> Result<bool, String> {
    let rows = sqlx::query(
        "SELECT id,text,source_segment_id,source_segment_revision,source_text \
         FROM spaces_speech_segments WHERE project_revision_id=? ORDER BY ordinal",
    )
    .bind(revision_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    if rows.len() != requested.len() {
        return Ok(false);
    }
    for (row, requested) in rows.into_iter().zip(requested) {
        let id: String = row.try_get("id").map_err(db::sql_error)?;
        let text: String = row.try_get("text").map_err(db::sql_error)?;
        let source_id: Option<String> = row.try_get("source_segment_id").map_err(db::sql_error)?;
        let source_revision: Option<i64> = row
            .try_get("source_segment_revision")
            .map_err(db::sql_error)?;
        let source_text: Option<String> = row.try_get("source_text").map_err(db::sql_error)?;
        let expected_id = source_id.as_deref().unwrap_or(&id);
        let source_changed = !translated
            && source_text
                .as_deref()
                .is_some_and(|source| source.trim() != text.trim());
        let expected_revision = source_revision
            .unwrap_or(1)
            .checked_add(if source_changed { 1 } else { 0 })
            .ok_or("Video source segment revision is exhausted")?;
        let expected_text = if translated {
            source_text
                .as_deref()
                .ok_or("Translated video segment source text is unavailable")?
        } else {
            &text
        };
        if requested.source_segment_id != expected_id
            || i64::from(requested.source_segment_revision) != expected_revision
            || requested.text.trim() != expected_text.trim()
        {
            return Ok(false);
        }
    }
    Ok(true)
}

pub(super) async fn existing_request(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: &str,
    expected_space_id: &str,
) -> Result<Option<TranslationRunInput>, String> {
    let row: Option<(String, Option<String>, String)> = sqlx::query_as(
        "SELECT scope_kind,home_space_id,input_snapshot FROM spaces_runs WHERE id=?",
    )
    .bind(run_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let Some((scope_kind, home_space_id, encoded)) = row else {
        return Ok(None);
    };
    let scope = RunScope::from_db(&scope_kind, home_space_id)?;
    let value: serde_json::Value = serde_json::from_str(&encoded).map_err(db::json_error)?;
    if scope
        != (RunScope::Space {
            space_id: expected_space_id.to_owned(),
        })
        || value.get("workflow").and_then(serde_json::Value::as_str) != Some(WORKFLOW)
    {
        return Err("request_conflict: translation request identity belongs to another Run".into());
    }
    Ok(Some(serde_json::from_value(value).map_err(db::json_error)?))
}

pub(super) async fn active_for_revision(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    project_id: &str,
    expected_revision: i64,
) -> Result<Option<(String, TranslationRunInput)>, String> {
    let row: Option<(String, String)> = sqlx::query_as(
        "SELECT id,input_snapshot FROM spaces_runs WHERE scope_kind='space' \
         AND home_space_id=? AND completed_at IS NULL \
         AND json_extract(input_snapshot,'$.workflow')=? \
         AND json_extract(input_snapshot,'$.projectId')=? \
         AND json_extract(input_snapshot,'$.expectedRevision')=? ORDER BY created_at DESC LIMIT 1",
    )
    .bind(space_id)
    .bind(WORKFLOW)
    .bind(project_id)
    .bind(expected_revision)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    row.map(|(id, encoded)| {
        serde_json::from_str(&encoded)
            .map(|input| (id, input))
            .map_err(db::json_error)
    })
    .transpose()
}

pub(super) async fn latest_recoverable_run(
    pool: &SqlitePool,
    project_id: &str,
    expected_revision: i64,
) -> Result<Option<String>, String> {
    sqlx::query_scalar(
        "SELECT r.id FROM spaces_runs r JOIN spaces_speech_projects p ON p.id=? \
         WHERE r.scope_kind='space' AND r.home_space_id=p.space_id AND \
         json_extract(r.input_snapshot,'$.workflow')=? \
         AND json_extract(r.input_snapshot,'$.projectId')=? \
         AND json_extract(r.input_snapshot,'$.expectedRevision')=? \
         AND (r.completed_at IS NULL OR r.state='succeeded') \
         ORDER BY CASE WHEN r.completed_at IS NULL THEN 0 ELSE 1 END,r.created_at DESC LIMIT 1",
    )
    .bind(project_id)
    .bind(WORKFLOW)
    .bind(project_id)
    .bind(expected_revision)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)
}

pub(super) async fn load_translation(
    pool: &SqlitePool,
    run_id: &str,
) -> Result<VideoTranslation, String> {
    let row: Option<(String, String)> = sqlx::query_as(
        "SELECT r.state,r.input_snapshot FROM spaces_runs r \
         JOIN spaces_speech_projects p \
         ON p.id=json_extract(r.input_snapshot,'$.projectId') \
         WHERE r.id=? AND r.scope_kind='space' AND r.home_space_id=p.space_id",
    )
    .bind(run_id)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?;
    let (state, encoded) = row.ok_or("Video translation Run is unavailable")?;
    if state != "succeeded" {
        return Err("resource_busy: video translation has not completed".into());
    }
    let input: TranslationRunInput = serde_json::from_str(&encoded).map_err(db::json_error)?;
    if input.workflow != WORKFLOW {
        return Err("Video translation Run identity differs".into());
    }
    let rows = sqlx::query(
        "SELECT id,configuration,output FROM spaces_run_steps WHERE run_id=? AND node_kind=? \
         AND state='succeeded' ORDER BY CAST(json_extract(configuration,'$.ordinal') AS INTEGER)",
    )
    .bind(run_id)
    .bind(super::NODE_KIND)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    if rows.len() != input.segments.len() {
        return Err("invalid_output: video translation checkpoints are incomplete".into());
    }
    let mut segments = Vec::with_capacity(rows.len());
    for (ordinal, row) in rows.into_iter().enumerate() {
        let config: TranslationStepConfiguration = serde_json::from_str(
            &row.try_get::<String, _>("configuration")
                .map_err(db::sql_error)?,
        )
        .map_err(db::json_error)?;
        let step_id: String = row.try_get("id").map_err(db::sql_error)?;
        let output: VideoTranslationSegmentOutput =
            serde_json::from_str(&row.try_get::<String, _>("output").map_err(db::sql_error)?)
                .map_err(db::json_error)?;
        let expected_segment = input
            .segments
            .get(ordinal)
            .ok_or("invalid_output: video translation checkpoint ordinal is invalid")?;
        if config.ordinal != ordinal
            || output.step_id != step_id
            || config.project_id != input.project_id
            || config.revision_id != input.revision_id
            || config.expected_revision != input.expected_revision
            || config.input_revision != input.input_revision
            || config.target_language != input.target_language
            || config.local_only != input.local_only
            || config.provider != input.provider
            || config.model != input.model
            || config.base_url != input.base_url
            || config.execution_target != input.execution_target
            || &config.segment != expected_segment
            || config.segment.source_segment_id != output.source_segment_id
            || config.segment.source_segment_revision != output.source_segment_revision
            || config.segment.text.trim() != output.source_text
        {
            return Err("invalid_output: video translation checkpoint identity differs".into());
        }
        segments.push(output);
    }
    Ok(VideoTranslation {
        provenance: super::super::video_config::VideoTranslationProvenance {
            run_id: run_id.to_owned(),
            target_language: input.target_language,
            provider: input.provider,
            model: input.model,
            local_only: input.local_only,
        },
        segments,
    })
}

fn require_canonical_uuid(value: &str, label: &str) -> Result<(), String> {
    let parsed = uuid::Uuid::parse_str(value).map_err(|_| format!("{label} must be a UUID"))?;
    if parsed.to_string() != value {
        return Err(format!("{label} must use canonical UUID form"));
    }
    Ok(())
}
