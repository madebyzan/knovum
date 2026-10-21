use tauri::{AppHandle, Runtime};

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
) -> AppResult<ModelExecutionTarget> {
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
    let enabled = settings
        .connections
        .iter()
        .find(|connection| connection.id == connection_id)
        .and_then(|connection| connection.models.iter().find(|model| model.id == model_id))
        .is_some_and(|model| model.builtin_agent);
    if !enabled {
        return Err(AppError::invalid(
            "Enable the model for the Builtin Agent in AI settings",
        ));
    }
    let target =
        super::resolve_model(settings, &connection_id, &model_id, temperature, capability)?;
    validate(target.model_provider_kind()?)?;
    Ok(target)
}

pub(super) fn validate(provider: ProviderKind) -> AppResult<()> {
    if crate::ai::providers::registry::builtin_agent_protocol(provider).is_none() {
        return Err(AppError::invalid(
            "The configured provider cannot run the Builtin Agent",
        ));
    }
    Ok(())
}
