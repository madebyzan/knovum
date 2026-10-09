use sqlx::{Row, Sqlite, Transaction};

use super::{
    contracts::{TranslationRunInput, TranslationStepConfiguration, VideoTranslationSegmentOutput},
    NODE_KIND, WORKFLOW,
};
use crate::spaces::{authorization::RunScope, db, speech::contracts::SpeechSegmentInput};

pub(crate) struct SaveContext<'a> {
    pub space_id: &'a str,
    pub project_id: &'a str,
    pub active_revision_id: &'a str,
    pub active_revision: i64,
    pub active_output_config: &'a serde_json::Value,
    pub requested_output_config: &'a super::super::video_config::VideoDubbingOutputConfig,
    pub segments: &'a [SpeechSegmentInput],
}

pub(crate) async fn validate_saved_provenance(
    tx: &mut Transaction<'_, Sqlite>,
    context: SaveContext<'_>,
) -> Result<(), String> {
    let Some(provenance) = &context.requested_output_config.translation else {
        return Ok(());
    };
    let row: Option<(String, String, Option<String>, String)> = sqlx::query_as(
        "SELECT state,scope_kind,home_space_id,input_snapshot FROM spaces_runs WHERE id=?",
    )
    .bind(&provenance.run_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let Some((state, scope_kind, home_space_id, encoded)) = row else {
        return Err("Video translation provenance Run is unavailable".into());
    };
    let scope = RunScope::from_db(&scope_kind, home_space_id)?;
    let input: TranslationRunInput = serde_json::from_str(&encoded).map_err(db::json_error)?;
    if state != "succeeded"
        || scope
            != (RunScope::Space {
                space_id: context.space_id.to_owned(),
            })
        || input.workflow != WORKFLOW
        || input.project_id != context.project_id
        || input.expected_revision > context.active_revision
        || provenance.target_language != input.target_language
        || provenance.provider != input.provider
        || provenance.model != input.model
        || provenance.local_only != input.local_only
    {
        return Err("Video translation provenance differs from its succeeded Run".into());
    }
    let base: Option<(String, i64, Option<String>)> = sqlx::query_as(
        "SELECT project_id,revision,json_type(output_config,'$.translation') \
         FROM spaces_speech_project_revisions WHERE id=?",
    )
    .bind(&input.revision_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let Some((base_project_id, base_revision, translation_type)) = base else {
        return Err("Video translation base revision is unavailable".into());
    };
    if base_project_id != context.project_id || base_revision != input.expected_revision {
        return Err("Video translation base revision differs from its Run snapshot".into());
    }
    super::repository::require_run_source_segments(
        tx,
        &input.revision_id,
        translation_type.as_deref() == Some("object"),
        &input.segments,
    )
    .await?;
    validate_outputs(tx, &provenance.run_id, &input).await?;
    validate_source_identity(context.segments, &input)?;
    if context.active_revision_id != input.revision_id {
        validate_successor(tx, context, &provenance.run_id, &input).await?;
    }
    Ok(())
}

async fn validate_outputs(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: &str,
    input: &TranslationRunInput,
) -> Result<(), String> {
    let rows = sqlx::query(
        "SELECT id,configuration,output FROM spaces_run_steps WHERE run_id=? AND node_kind=? \
         AND state='succeeded' ORDER BY CAST(json_extract(configuration,'$.ordinal') AS INTEGER)",
    )
    .bind(run_id)
    .bind(NODE_KIND)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    if rows.len() != input.segments.len() {
        return Err("Video translation provenance checkpoints are incomplete".into());
    }
    for (ordinal, row) in rows.into_iter().enumerate() {
        let step_id: String = row.try_get("id").map_err(db::sql_error)?;
        let config: TranslationStepConfiguration = serde_json::from_str(
            &row.try_get::<String, _>("configuration")
                .map_err(db::sql_error)?,
        )
        .map_err(db::json_error)?;
        let output: VideoTranslationSegmentOutput =
            serde_json::from_str(&row.try_get::<String, _>("output").map_err(db::sql_error)?)
                .map_err(db::json_error)?;
        let expected = input
            .segments
            .get(ordinal)
            .ok_or("Video translation provenance checkpoint ordinal is invalid")?;
        if config.ordinal != ordinal
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
            || &config.segment != expected
            || output.step_id != step_id
            || output.source_segment_id != expected.source_segment_id
            || output.source_segment_revision != expected.source_segment_revision
            || output.source_text != expected.text.trim()
        {
            return Err("Video translation provenance checkpoint identity differs".into());
        }
    }
    Ok(())
}

fn validate_source_identity(
    segments: &[SpeechSegmentInput],
    input: &TranslationRunInput,
) -> Result<(), String> {
    if segments.len() != input.segments.len()
        || segments.iter().zip(&input.segments).any(|(saved, source)| {
            saved.source_segment_id.as_deref() != Some(source.source_segment_id.as_str())
                || saved.source_segment_revision != Some(source.source_segment_revision)
                || saved.source_text.as_deref().map(str::trim) != Some(source.text.trim())
        })
    {
        return Err("Translated video segments differ from their Run source identity".into());
    }
    Ok(())
}

async fn validate_successor(
    tx: &mut Transaction<'_, Sqlite>,
    context: SaveContext<'_>,
    run_id: &str,
    input: &TranslationRunInput,
) -> Result<(), String> {
    if context.active_revision <= input.expected_revision {
        return Err(
            "Video translation provenance is not based on the active revision lineage".into(),
        );
    }
    let active = super::super::video_config::parse(context.active_output_config)?;
    let active_provenance = active
        .translation
        .as_ref()
        .ok_or("Active translated revision has no Run provenance")?;
    if active_provenance.run_id != run_id
        || active_provenance.target_language != input.target_language
        || active_provenance.provider != input.provider
        || active_provenance.model != input.model
        || active_provenance.local_only != input.local_only
    {
        return Err("Edited translated revision does not preserve its Run provenance".into());
    }
    let rows: Vec<(Option<String>, Option<i64>, Option<String>)> = sqlx::query_as(
        "SELECT source_segment_id,source_segment_revision,source_text FROM spaces_speech_segments \
         WHERE project_revision_id=? ORDER BY ordinal",
    )
    .bind(context.active_revision_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    if rows.len() != input.segments.len()
        || rows.iter().zip(&input.segments).any(|(saved, source)| {
            saved.0.as_deref() != Some(source.source_segment_id.as_str())
                || saved.1 != Some(i64::from(source.source_segment_revision))
                || saved.2.as_deref().map(str::trim) != Some(source.text.trim())
        })
    {
        return Err("Active translated revision differs from its Run source identity".into());
    }
    Ok(())
}
