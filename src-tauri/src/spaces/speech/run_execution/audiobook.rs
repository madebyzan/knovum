use serde::Deserialize;
use sqlx::Row;

use super::super::{
    contracts::{SpeechProjectRenderOutput, SpeechSegmentRenderOutput},
    execution::command_error,
    project_publication, publication,
};
use crate::{
    foundation::app_error::AppError,
    spaces::{
        db,
        runs::{AttemptClaim, StepOutcome},
    },
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ChapterConfiguration {
    project_id: String,
    revision_id: String,
    revision: i64,
    chapter_key: String,
    #[serde(rename = "chapterIndex")]
    _chapter_index: usize,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AudiobookStageConfiguration {
    project_id: String,
    revision_id: String,
    revision: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AssemblyConfiguration {
    project_id: String,
    revision_id: String,
    revision: i64,
}

pub(super) async fn execute_chapter(
    app: &tauri::AppHandle,
    pool: &sqlx::SqlitePool,
    claim: &AttemptClaim,
) -> Result<StepOutcome, AppError> {
    let config: ChapterConfiguration =
        serde_json::from_value(claim.configuration.clone()).map_err(db::json_error)?;
    let output_config = require_audiobook_revision(
        pool,
        claim,
        &config.project_id,
        &config.revision_id,
        config.revision,
    )
    .await?;
    let audiobook = super::super::project_config::parse_audiobook(&output_config)?;
    let rows = sqlx::query(
        "SELECT rs.output FROM spaces_speech_segments s JOIN spaces_run_steps rs \
         ON rs.run_id=? AND rs.definition_node_id='segment:'||s.id \
         WHERE s.project_revision_id=? AND s.chapter_key=? AND rs.state='succeeded' \
         ORDER BY s.ordinal",
    )
    .bind(&claim.run_id)
    .bind(&config.revision_id)
    .bind(&config.chapter_key)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    if rows.is_empty() {
        return Err("Audiobook chapter has no completed segments".into());
    }
    let mut renders = Vec::with_capacity(rows.len());
    for row in rows {
        let encoded: String = row.try_get("output").map_err(db::sql_error)?;
        let output: SpeechSegmentRenderOutput =
            serde_json::from_str(&encoded).map_err(db::json_error)?;
        renders.push(publication::validate_cached(app, output)?);
    }
    let checkpoint = project_publication::publish_chapter_checkpoint(
        app,
        pool,
        claim,
        &project_publication::ChapterPublicationRequest {
            project_id: &config.project_id,
            revision_id: &config.revision_id,
            chapter_key: &config.chapter_key,
            sample_rate: audiobook.sample_rate,
            paragraph_pause_ms: audiobook.paragraph_pause_ms,
            renders: &renders,
        },
        &claim.cancellation,
    )
    .await?;
    Ok(StepOutcome::Completed {
        output: serde_json::to_value(checkpoint).map_err(db::json_error)?,
        selected_branch: None,
    })
}

pub(super) async fn execute_loudness(
    app: &tauri::AppHandle,
    pool: &sqlx::SqlitePool,
    claim: &AttemptClaim,
) -> Result<StepOutcome, AppError> {
    let config: AudiobookStageConfiguration =
        serde_json::from_value(claim.configuration.clone()).map_err(db::json_error)?;
    let output_config = require_audiobook_revision(
        pool,
        claim,
        &config.project_id,
        &config.revision_id,
        config.revision,
    )
    .await?;
    let chapters = load_chapter_outputs(pool, &claim.run_id).await?;
    let input = project_publication::AudiobookInput {
        project_id: &config.project_id,
        revision_id: &config.revision_id,
        output_config: &output_config,
        chapters: &chapters,
    };
    let measurement =
        project_publication::measure_audiobook(app, pool, claim, &input, &claim.cancellation)
            .await?;
    Ok(StepOutcome::Completed {
        output: serde_json::to_value(measurement).map_err(db::json_error)?,
        selected_branch: None,
    })
}

pub(super) async fn execute_audiobook(
    app: &tauri::AppHandle,
    pool: &sqlx::SqlitePool,
    claim: &AttemptClaim,
) -> Result<StepOutcome, AppError> {
    let config: AssemblyConfiguration =
        serde_json::from_value(claim.configuration.clone()).map_err(db::json_error)?;
    let output_config = require_audiobook_revision(
        pool,
        claim,
        &config.project_id,
        &config.revision_id,
        config.revision,
    )
    .await?;
    let chapters = load_chapter_outputs(pool, &claim.run_id).await?;
    let encoded: String = sqlx::query_scalar(
        "SELECT output FROM spaces_run_steps WHERE run_id=? \
         AND node_kind='speech_loudness_measurement' AND state='succeeded'",
    )
    .bind(&claim.run_id)
    .fetch_one(pool)
    .await
    .map_err(db::sql_error)?;
    let measurement: project_publication::LoudnessCheckpoint =
        serde_json::from_str(&encoded).map_err(db::json_error)?;
    let input = project_publication::AudiobookInput {
        project_id: &config.project_id,
        revision_id: &config.revision_id,
        output_config: &output_config,
        chapters: &chapters,
    };
    let output = project_publication::publish_audiobook(
        app,
        pool,
        claim,
        &input,
        measurement,
        &claim.cancellation,
    )
    .await?;
    if let Err(error) = project_publication::commit_project_output(
        pool,
        claim,
        &config.project_id,
        &config.revision_id,
        &output,
        None,
        crate::spaces::assets::enrichment::jobs::intent::selection(app)?.as_ref(),
    )
    .await
    {
        super::super::asset_publication::abandon_attempt(app, pool, claim).await?;
        return Err(command_error(error, claim.cancellation.is_cancelled()));
    }
    project_publication::cleanup_audiobook_checkpoints(app, pool, &config.revision_id).await?;
    let renders = load_segment_outputs(app, pool, &claim.run_id, &config.revision_id).await?;
    Ok(StepOutcome::Completed {
        output: serde_json::to_value(SpeechProjectRenderOutput {
            project_id: config.project_id,
            revision: config.revision,
            renders,
            output,
        })
        .map_err(db::json_error)?,
        selected_branch: None,
    })
}

async fn require_audiobook_revision(
    pool: &sqlx::SqlitePool,
    claim: &AttemptClaim,
    project_id: &str,
    revision_id: &str,
    revision: i64,
) -> Result<serde_json::Value, String> {
    let run_space_id = claim.scope.require_space_id()?;
    let row: Option<(String, i64, String, String)> = sqlx::query_as(
        "SELECT p.active_revision_id,r.revision,r.output_config,p.space_id \
         FROM spaces_speech_projects p JOIN spaces_speech_project_revisions r \
         ON r.id=p.active_revision_id WHERE p.id=? AND r.id=? AND p.project_type='audiobook'",
    )
    .bind(project_id)
    .bind(revision_id)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?;
    let Some((active_revision, actual_revision, output_config, space_id)) = row else {
        return Err("stale_revision: audiobook project changed before execution".into());
    };
    if active_revision != revision_id
        || actual_revision != revision
        || space_id.as_str() != run_space_id
    {
        return Err("stale_revision: audiobook project changed before execution".into());
    }
    serde_json::from_str(&output_config).map_err(db::json_error)
}

async fn load_chapter_outputs(
    pool: &sqlx::SqlitePool,
    run_id: &str,
) -> Result<Vec<project_publication::ChapterCheckpoint>, String> {
    let outputs: Vec<String> = sqlx::query_scalar(
        "SELECT output FROM spaces_run_steps WHERE run_id=? AND node_kind='speech_chapter_pcm' \
         AND state='succeeded' ORDER BY CAST(json_extract(configuration,'$.chapterIndex') AS INTEGER)",
    ).bind(run_id).fetch_all(pool).await.map_err(db::sql_error)?;
    if outputs.is_empty() {
        return Err("Audiobook has no completed chapter checkpoints".into());
    }
    outputs
        .into_iter()
        .map(|encoded| serde_json::from_str(&encoded).map_err(db::json_error))
        .collect()
}

async fn load_segment_outputs(
    app: &tauri::AppHandle,
    pool: &sqlx::SqlitePool,
    run_id: &str,
    revision_id: &str,
) -> Result<Vec<SpeechSegmentRenderOutput>, String> {
    let outputs: Vec<String> = sqlx::query_scalar(
        "SELECT rs.output FROM spaces_speech_segments s JOIN spaces_run_steps rs \
         ON rs.run_id=? AND rs.definition_node_id='segment:'||s.id \
         WHERE s.project_revision_id=? AND rs.state='succeeded' ORDER BY s.ordinal",
    )
    .bind(run_id)
    .bind(revision_id)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    let mut renders = Vec::with_capacity(outputs.len());
    for encoded in outputs {
        let output = serde_json::from_str(&encoded).map_err(db::json_error)?;
        renders.push(publication::validate_cached(app, output)?);
    }
    Ok(renders)
}
