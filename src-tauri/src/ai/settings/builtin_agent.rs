use tauri::{AppHandle, Runtime};

use super::contracts::BuiltinAgentModelSettings;
use crate::{
    ai::{
        contracts::{ExecutionCapability, ProviderKind},
        models::{ModelExecutionTarget, ModelTargetSelection},
    },
    app_error::{AppError, AppResult},
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

pub(super) fn validate(
    provider: ProviderKind,
    profile: &BuiltinAgentModelSettings,
) -> AppResult<()> {
    if !(8192..=2_000_000).contains(&profile.context_window)
        || profile.max_output_tokens == 0
        || profile.max_output_tokens >= profile.context_window
    {
        return Err(AppError::invalid(
            "Builtin Agent context and output limits are invalid",
        ));
    }
    let allowed = crate::ai::providers::registry::descriptor(provider)
        .is_some_and(|value| value.builtin_agent_protocols.contains(&profile.protocol));
    if !allowed {
        return Err(AppError::invalid(
            "Builtin Agent API protocol does not match the configured provider",
        ));
    }
    Ok(())
}
