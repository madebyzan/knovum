use synthesis::{synthesize, validate_segment_timing, SegmentTiming};
mod synthesis;
use crate::spaces::speech::asset_publication::PublicationOwnerKind;

use serde::Deserialize;
use sqlx::Row;

use super::super::{
    contracts::{ProjectRevisionIdentity, SpeechSegmentExecution, SpeechSegmentRenderOutput},
    execution::{command_error, input_digest, load_voice, parse_config, SynthesisAttempt},
    generation_attempt, publication,
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
struct SegmentConfiguration {
    project_id: String,
    revision_id: String,
    segment_id: String,
    ordinal: i64,
}

pub(super) async fn execute(
    app: &tauri::AppHandle,
    pool: &sqlx::SqlitePool,
    claim: &AttemptClaim,
) -> Result<StepOutcome, AppError> {
    let run_space_id = claim.scope.require_space_id()?;
    let config: SegmentConfiguration =
        serde_json::from_value(claim.configuration.clone()).map_err(db::json_error)?;
    let input: Option<String> =
        sqlx::query_scalar("SELECT input_snapshot FROM spaces_runs WHERE id=?")
            .bind(&claim.run_id)
            .fetch_optional(pool)
            .await
            .map_err(db::sql_error)?;
    let input: serde_json::Value = serde_json::from_str(
        &input.ok_or("runtime_mismatch: Speech Run input snapshot is unavailable")?,
    )
    .map_err(db::json_error)?;
    if input.get("workflow").and_then(serde_json::Value::as_str)
        != Some("speech_project_generation_v1")
    {
        return Err("runtime_mismatch: Speech segment belongs to another workflow".into());
    }
    let frozen_runtimes = super::super::runtime_identity::read_snapshot(&input)?;
    uuid::Uuid::parse_str(&config.project_id).map_err(|_| "Speech project identity is invalid")?;
    uuid::Uuid::parse_str(&config.revision_id)
        .map_err(|_| "Speech revision identity is invalid")?;
    uuid::Uuid::parse_str(&config.segment_id).map_err(|_| "Speech segment identity is invalid")?;
    let row = sqlx::query(
        "SELECT p.space_id,p.project_type,s.id,s.normalized_text,s.delivery_instruction,s.voice_revision_id,s.language, \
         s.target_start_ms,s.target_end_ms FROM spaces_speech_projects p \
         JOIN spaces_speech_segments s ON s.project_revision_id=p.active_revision_id \
         WHERE p.id=? AND p.active_revision_id=? AND s.id=? AND s.ordinal=?",
    )
    .bind(&config.project_id)
    .bind(&config.revision_id)
    .bind(&config.segment_id)
    .bind(config.ordinal)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?
    .ok_or("stale_revision: speech segment is no longer active")?;
    let segment = SpeechSegmentExecution {
        id: row.try_get("id").map_err(db::sql_error)?,
        normalized_text: row.try_get("normalized_text").map_err(db::sql_error)?,
        delivery_instruction: row.try_get("delivery_instruction").map_err(db::sql_error)?,
        voice_revision_id: row.try_get("voice_revision_id").map_err(db::sql_error)?,
        language: row.try_get("language").map_err(db::sql_error)?,
    };
    let space_id: String = row.try_get("space_id").map_err(db::sql_error)?;
    let timing = SegmentTiming {
        project_type: row.try_get("project_type").map_err(db::sql_error)?,
        start_ms: row.try_get("target_start_ms").map_err(db::sql_error)?,
        end_ms: row.try_get("target_end_ms").map_err(db::sql_error)?,
    };
    if space_id.as_str() != run_space_id {
        return Err("permission-denied: Speech project belongs to another Space".into());
    }
    let voice = load_voice(pool, &segment.voice_revision_id).await?;
    let profile = super::super::runtime_identity::profile_for_engine(&voice.engine_id)?;
    let runtime_digest = crate::local_resources::runtimes::inference::resolve(app)?.digest;
    if frozen_runtimes.get(&profile) != Some(&runtime_digest) {
        return Err("runtime_mismatch: Speech Run runtime package changed".into());
    }
    if voice.language != segment.language {
        return Err("Speech segment language differs from its voice revision".into());
    }
    let voice_config = parse_config(&voice.synthesis_config)?;
    let device_profile = super::super::execution::synthesis_device_profile(
        app,
        &voice.engine_id,
        &voice.model_spec_id,
        &claim.cancellation,
    )
    .await?;
    let digest = input_digest(
        &segment,
        &voice,
        &voice_config,
        &runtime_digest,
        &device_profile.execution_profile_id,
    )?;
    if let Some(cached) =
        generation_attempt::cached_render(pool, &config.project_id, &segment.id, &digest).await?
    {
        match publication::validate_cached(
            app,
            generation_attempt::cached_output(&segment, &cached)?,
        ) {
            Ok(output) => {
                if let Err(reason) =
                    validate_segment_timing(&timing, output.sample_rate, output.sample_count)
                {
                    let reason = format!("duration_out_of_range: {reason}");
                    generation_attempt::mark_segment_needs_edit(
                        pool,
                        claim,
                        &config.project_id,
                        &config.revision_id,
                        &segment.id,
                        &reason,
                    )
                    .await?;
                    return Err(AppError::new("duration_out_of_range", reason));
                }
                generation_attempt::publish_cached(
                    pool,
                    claim,
                    ProjectRevisionIdentity {
                        project_id: &config.project_id,
                        revision_id: &config.revision_id,
                    },
                    &segment,
                    &voice,
                    &digest,
                    &cached,
                )
                .await
                .map_err(|error| command_error(error, claim.cancellation.is_cancelled()))?;
                return completed(output);
            }
            Err(_) => generation_attempt::reject_corrupt_cache(pool, &cached).await?,
        }
    }
    let target_path = publication::segment_path(
        app,
        &config.project_id,
        &config.revision_id,
        &segment.id,
        &digest,
        &claim.attempt_id,
    )?;
    let (render_id, attempt_epoch) = generation_attempt::stage_render(
        pool,
        claim,
        ProjectRevisionIdentity {
            project_id: &config.project_id,
            revision_id: &config.revision_id,
        },
        &segment,
        &voice,
        &digest,
        &target_path,
    )
    .await?;
    let audio = synthesize(
        app,
        claim,
        &voice,
        &segment,
        SynthesisAttempt {
            request_id: &claim.run_id,
            owner_kind: "speech_project_segment",
            owner_id: &segment.id,
            input_revision: &digest,
            attempt_id: &claim.attempt_id,
            attempt_epoch,
        },
    )
    .await;
    let audio = match audio {
        Ok(audio) => audio,
        Err(error) => {
            generation_attempt::fail_render(
                pool,
                claim,
                &render_id,
                &segment.id,
                &error,
                claim.cancellation.is_cancelled(),
            )
            .await?;
            return Err(command_error(error, claim.cancellation.is_cancelled()));
        }
    };
    if let Err(error) = publication::validate_synthesized_audio(&audio) {
        generation_attempt::fail_render(pool, claim, &render_id, &segment.id, &error, false)
            .await?;
        return Err(error.into());
    }
    if audio.sample_rate != voice_config.sample_rate {
        let error = "invalid_output: synthesized sample rate differs from the voice revision";
        generation_attempt::fail_render(pool, claim, &render_id, &segment.id, error, false).await?;
        return Err(error.into());
    }
    if let Err(reason) = validate_segment_timing(&timing, audio.sample_rate, audio.sample_count) {
        let reason = format!("duration_out_of_range: {reason}");
        generation_attempt::mark_needs_edit(pool, claim, &render_id, &segment.id, &reason).await?;
        return Err(AppError::new("duration_out_of_range", reason));
    }
    let part_path = target_path.with_extension("wav.part");
    let publication_id = match super::super::asset_publication::stage_for_attempt(
        pool,
        claim,
        super::super::asset_publication::PendingAsset {
            owner_kind: PublicationOwnerKind::SegmentRender,
            owner_id: &render_id,
            asset_path: &target_path,
            part_path: Some(&part_path),
            sha256: &audio.sha256,
        },
    )
    .await
    {
        Ok(id) => id,
        Err(error) => {
            generation_attempt::fail_render(
                pool,
                claim,
                &render_id,
                &segment.id,
                &error,
                claim.cancellation.is_cancelled(),
            )
            .await?;
            return Err(command_error(error, claim.cancellation.is_cancelled()));
        }
    };
    let path = match publication::publish_segment(&target_path, &audio) {
        Ok(path) => path,
        Err(error) => {
            super::super::asset_publication::abandon(app, pool, &publication_id).await?;
            generation_attempt::fail_render(pool, claim, &render_id, &segment.id, &error, false)
                .await?;
            return Err(error.into());
        }
    };
    if let Err(error) = publication::commit_render(
        pool,
        claim,
        ProjectRevisionIdentity {
            project_id: &config.project_id,
            revision_id: &config.revision_id,
        },
        &render_id,
        &segment.id,
        &path,
        &audio,
    )
    .await
    {
        super::super::asset_publication::abandon(app, pool, &publication_id).await?;
        generation_attempt::fail_render(
            pool,
            claim,
            &render_id,
            &segment.id,
            &error,
            claim.cancellation.is_cancelled(),
        )
        .await?;
        return Err(command_error(error, claim.cancellation.is_cancelled()));
    }
    completed(SpeechSegmentRenderOutput {
        segment_id: segment.id,
        path: path.to_string_lossy().into_owned(),
        sha256: audio.sha256,
        sample_rate: audio.sample_rate,
        sample_count: audio.sample_count,
        cached: false,
    })
}

fn completed(output: SpeechSegmentRenderOutput) -> Result<StepOutcome, AppError> {
    Ok(StepOutcome::Completed {
        output: serde_json::to_value(output).map_err(db::json_error)?,
        selected_branch: None,
    })
}
