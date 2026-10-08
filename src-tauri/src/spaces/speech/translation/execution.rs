use crate::{
    ai::{
        capabilities,
        contracts::ExecutionCapability,
        settings,
        tasks::{self, TaskExecutionTarget},
    },
    app_error::AppError,
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
    let config: TranslationStepConfiguration =
        serde_json::from_value(claim.configuration.clone()).map_err(db::json_error)?;
    let mut tx = pool.begin().await.map_err(db::sql_error)?;
    let project = repository::active_project(&mut tx, &config.project_id).await?;
    repository::require_expected_revision(&project, config.expected_revision)?;
    if project.space_id != claim.home_space_id || project.revision_id != config.revision_id {
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
    if config.local_only && !target.is_local() {
        return Err("stale_revision: local-only translation target is no longer local".into());
    }
    let output = tasks::completion::complete(
        app,
        TaskExecutionTarget::Model(target),
        capabilities::translation::input(
            config.segment.text.clone(),
            config.target_language.clone(),
        )?,
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
    let translation = VideoTranslationSegmentOutput {
        step_id: claim.step_instance_id.clone(),
        source_segment_id: config.segment.source_segment_id,
        source_segment_revision: config.segment.source_segment_revision,
        source_text: config.segment.text.trim().to_owned(),
        translated_text: translated_text.to_owned(),
    };
    Ok(StepOutcome::Completed {
        output: serde_json::to_value(translation).map_err(db::json_error)?,
        selected_branch: None,
    })
}
