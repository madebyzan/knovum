use crate::{
    ai::{
        capabilities::{
            self,
            translation::speech_rate::{estimate_duration_ms, requires_reduction},
        },
        contracts::ExecutionCapability,
        models::ModelExecutionTarget,
        settings,
        tasks::{self, completion::TaskInput, TaskExecutionTarget},
    },
    foundation::app_error::AppError,
    spaces::{
        db,
        runs::{AttemptClaim, StepOutcome},
    },
};

use super::{
    contracts::{TranslationStepConfiguration, VideoTranslationSegmentOutput},
    repository,
};

pub(crate) async fn execute(
    app: &tauri::AppHandle,
    pool: &sqlx::SqlitePool,
    claim: &AttemptClaim,
) -> Result<StepOutcome, AppError> {
    super::super::run_execution::require_authority(pool, claim).await?;
    let space_id = claim.scope.require_space_id()?;
    let config: TranslationStepConfiguration =
        serde_json::from_value(claim.configuration.clone()).map_err(db::json_error)?;
    let mut tx = pool.begin().await.map_err(db::sql_error)?;
    let project = repository::active_project(&mut tx, &config.project_id).await?;
    repository::require_expected_revision(&project, config.expected_revision)?;
    if project.space_id != space_id || project.revision_id != config.revision_id {
        return Err("stale_revision: video translation project identity changed".into());
    }
    tx.rollback().await.map_err(db::sql_error)?;
    let mut target = settings::resolve(
        app,
        config.execution_target.clone(),
        ExecutionCapability::TextGeneration,
    )?;
    target.temperature = 0.0;
    if target.provider() != config.provider
        || target.model() != config.model
        || target.base_url != config.base_url
    {
        return Err("stale_revision: video translation model configuration changed".into());
    }
    let translated_text = complete_once(
        app,
        pool,
        claim,
        &target,
        &config,
        capabilities::translation::dubbing_input(
            config.segment.text.clone(),
            config.target_language.clone(),
            config.segment.target_duration_ms,
        )?,
    )
    .await?;
    let translated_text =
        reduce_to_fit(app, pool, claim, &target, &config, translated_text).await?;
    let translation = VideoTranslationSegmentOutput {
        step_id: claim.step_instance_id.clone(),
        source_segment_id: config.segment.source_segment_id,
        source_segment_revision: config.segment.source_segment_revision,
        source_text: config.segment.text.trim().to_owned(),
        translated_text,
    };
    Ok(StepOutcome::Completed {
        output: serde_json::to_value(translation).map_err(db::json_error)?,
        selected_branch: None,
    })
}

/// When the adaptation is estimated to clearly overrun its physical window, spends one bounded
/// shortening round. The reduction is accepted only when it estimates shorter; a failure
/// propagates so the Run's own attempt policy retries the whole step rather than silently keeping
/// an overrun. Lines whose duration fits, or whose language has no rate model, keep the first
/// adaptation untouched.
async fn reduce_to_fit(
    app: &tauri::AppHandle,
    pool: &sqlx::SqlitePool,
    claim: &AttemptClaim,
    target: &ModelExecutionTarget,
    config: &TranslationStepConfiguration,
    translated_text: String,
) -> Result<String, AppError> {
    let Some(window_ms) = config.segment.target_duration_ms.filter(|ms| *ms > 0) else {
        return Ok(translated_text);
    };
    let Some(estimate) = estimate_duration_ms(&translated_text, &config.target_language) else {
        return Ok(translated_text);
    };
    if !requires_reduction(estimate, window_ms) {
        return Ok(translated_text);
    }
    let shortened = complete_once(
        app,
        pool,
        claim,
        target,
        config,
        capabilities::translation::dubbing_reduction_input(
            config.segment.text.clone(),
            config.target_language.clone(),
            window_ms,
            &translated_text,
            estimate,
        )?,
    )
    .await?;
    let shortened_estimate =
        estimate_duration_ms(&shortened, &config.target_language).unwrap_or(u64::MAX);
    Ok(if shortened_estimate < estimate {
        shortened
    } else {
        translated_text
    })
}

async fn complete_once(
    app: &tauri::AppHandle,
    pool: &sqlx::SqlitePool,
    claim: &AttemptClaim,
    target: &ModelExecutionTarget,
    config: &TranslationStepConfiguration,
    input: TaskInput,
) -> Result<String, AppError> {
    let output = tasks::completion::complete(
        app,
        TaskExecutionTarget::Model(target.clone()),
        input,
        claim.cancellation.clone(),
    )
    .await?;
    match &output.execution_source {
        tasks::ExecutionSource::Model { provider, model }
            if provider == &config.provider && model == &config.model => {}
        _ => return Err("invalid_output: video translation execution source differs".into()),
    }
    if let Some(usage) = &output.usage {
        crate::spaces::runs::record_usage(
            pool,
            &claim.run_id,
            usage.prompt_tokens,
            usage.completion_tokens,
            usage.total_tokens,
        )
        .await?;
    }
    let translated_text = output.text.trim();
    if translated_text.is_empty() || translated_text.chars().count() > 10_000 {
        return Err("Translated speech segment must contain between 1 and 10000 characters".into());
    }
    Ok(translated_text.to_owned())
}
