use serde::Deserialize;
use sqlx::Row;

use super::super::{
    contracts::{SpeechProjectRenderOutput, SpeechSegmentRenderOutput},
    execution::command_error,
    project_publication, publication,
};
use crate::{
    app_error::AppError,
    spaces::{
        db,
        runs::{AttemptClaim, StepOutcome},
    },
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AssemblyConfiguration {
    project_id: String,
    revision_id: String,
    revision: i64,
    project_type: String,
}

pub(super) async fn execute(
    app: &tauri::AppHandle,
    pool: &sqlx::SqlitePool,
    claim: &AttemptClaim,
) -> Result<StepOutcome, AppError> {
    let run_space_id = claim.scope.require_space_id()?;
    let config: AssemblyConfiguration =
        serde_json::from_value(claim.configuration.clone()).map_err(db::json_error)?;
    let revision: Option<(String, String, i64, String, String)> = sqlx::query_as(
        "SELECT p.active_revision_id,r.output_config,r.revision,p.project_type,p.space_id          FROM spaces_speech_projects p JOIN spaces_speech_project_revisions r          ON r.id=p.active_revision_id WHERE p.id=? AND r.id=?",
    ).bind(&config.project_id).bind(&config.revision_id)
    .fetch_optional(pool).await.map_err(db::sql_error)?;
    let Some((active_revision, output_config, revision_number, project_type, space_id)) = revision
    else {
        return Err("stale_revision: speech project changed before assembly".into());
    };
    if active_revision != config.revision_id
        || revision_number != config.revision
        || project_type != config.project_type
        || space_id.as_str() != run_space_id
    {
        return Err("stale_revision: speech project changed before assembly".into());
    }
    let rows = sqlx::query(
        "SELECT s.id,s.ordinal,s.chapter_key,s.target_start_ms,s.target_end_ms,rs.output \
         FROM spaces_speech_segments s JOIN spaces_run_steps rs \
         ON rs.run_id=? AND rs.definition_node_id='segment:'||s.id \
         WHERE s.project_revision_id=? AND rs.state='succeeded' ORDER BY s.ordinal",
    )
    .bind(&claim.run_id)
    .bind(&config.revision_id)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    let expected: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM spaces_speech_segments WHERE project_revision_id=?",
    )
    .bind(&config.revision_id)
    .fetch_one(pool)
    .await
    .map_err(db::sql_error)?;
    if i64::try_from(rows.len()).map_err(|_| "Speech segment count exceeds SQLite range")?
        != expected
    {
        return Err("Speech assembly is missing completed segment outputs".into());
    }
    let mut renders = Vec::with_capacity(rows.len());
    let mut chapter_keys = Vec::with_capacity(rows.len());
    let mut video_windows = Vec::with_capacity(rows.len());
    for row in rows {
        let encoded: String = row.try_get("output").map_err(db::sql_error)?;
        let output: SpeechSegmentRenderOutput =
            serde_json::from_str(&encoded).map_err(db::json_error)?;
        renders.push(publication::validate_cached(app, output)?);
        chapter_keys.push(
            row.try_get::<Option<String>, _>("chapter_key")
                .map_err(db::sql_error)?,
        );
        video_windows.push((
            row.try_get::<Option<i64>, _>("target_start_ms")
                .map_err(db::sql_error)?,
            row.try_get::<Option<i64>, _>("target_end_ms")
                .map_err(db::sql_error)?,
        ));
    }
    let output_config: serde_json::Value =
        serde_json::from_str(&output_config).map_err(db::json_error)?;
    let output = if config.project_type == "video_dubbing" {
        super::super::video_publication::publish_video(
            app,
            pool,
            claim,
            super::super::video_publication::VideoPublicationInput {
                project_id: &config.project_id,
                revision_id: &config.revision_id,
                output_config: &output_config,
                windows: &video_windows,
                renders: &renders,
            },
            &claim.cancellation,
        )
        .await?
    } else {
        project_publication::publish_project(
            app,
            pool,
            claim,
            &project_publication::ProjectOutputRequest {
                project_id: &config.project_id,
                revision_id: &config.revision_id,
                project_type: &config.project_type,
                output_config: &output_config,
                chapter_keys: &chapter_keys,
                renders: &renders,
            },
            &claim.cancellation,
        )
        .await?
    };
    if let Err(error) = project_publication::commit_project_output(
        pool,
        claim,
        &config.project_id,
        &config.revision_id,
        &output,
    )
    .await
    {
        super::super::asset_publication::abandon_attempt(app, pool, claim).await?;
        return Err(command_error(error, claim.cancellation.is_cancelled()));
    }
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
