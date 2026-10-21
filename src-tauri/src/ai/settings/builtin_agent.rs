use tauri::{AppHandle, Runtime};

use super::contracts::BuiltinAgentModelSettings;
use crate::{
    ai::{
        contracts::{ExecutionCapability, ProviderKind},
        models::{ModelExecutionTarget, ModelTargetSelection},
    },
    foundation::app_error::{AppError, AppResult},
};

pub(crate) fn resolve<R: Runtime>(
    app: &AppHandle<R>,
    selection: ModelTargetSelection,
    capability: ExecutionCapability,
) -> AppResult<(ModelExecutionTarget, BuiltinAgentModelSettings)> {
    let settings = super::load(app)?;
    let ModelTargetSelection::Model {
        connection_id,
        model_id,
        temperature,
    } = selection;
    if !temperature.is_finite() || !(0.0..=2.0).contains(&temperature) {
        return Err(AppError::invalid(
            "Model temperature must be between 0 and 2",
        ));
    }
    let profile = settings
        .connections
        .iter()
        .find(|connection| connection.id == connection_id)
        .and_then(|connection| connection.models.iter().find(|model| model.id == model_id))
        .and_then(|model| model.builtin_agent.clone())
        .ok_or_else(|| {
            AppError::invalid(
                "Enable tool calling and configure the Builtin Agent model profile in AI settings",
            )
        })?;
    let target =
        super::resolve_model(settings, &connection_id, &model_id, temperature, capability)?;
    validate(target.model_provider_kind()?, &profile)?;
    Ok((target, profile))
}

/// Input the Builtin Agent needs besides one answer: its harness prompt, tool
/// schemas and the Space snapshot (about 12k tokens) must stay well under half
/// of the input, or context compaction can never get below its own trigger.
pub(crate) const MIN_INPUT_TOKENS: u32 = 32_768;
const MAX_CONTEXT_WINDOW: u32 = 2_000_000;

pub(super) fn validate(
    provider: ProviderKind,
    profile: &BuiltinAgentModelSettings,
) -> AppResult<()> {
    if profile.context_window > MAX_CONTEXT_WINDOW
        || profile.max_output_tokens == 0
        || profile
            .context_window
            .saturating_sub(profile.max_output_tokens)
            < MIN_INPUT_TOKENS
    {
        return Err(AppError::invalid(
            "Builtin Agent context and output limits are invalid",
        ));
    }
    if crate::ai::providers::registry::builtin_agent_protocol(provider).is_none() {
        return Err(AppError::invalid(
            "The configured provider cannot run the Builtin Agent",
        ));
    }
    Ok(())
}
