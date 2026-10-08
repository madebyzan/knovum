use std::collections::HashSet;

use crate::ai::{contracts::RoutingPurpose, tasks::TaskTargetRef};
use crate::app_error::{AppError, AppResult};

use super::contracts::AiSettings;

pub(super) fn validate(settings: &AiSettings) -> AppResult<()> {
    let mut connection_ids = HashSet::new();
    for connection in &settings.connections {
        if connection.id.trim().is_empty()
            || connection.name.trim().is_empty()
            || connection.base_url.trim().is_empty()
            || !connection_ids.insert(connection.id.as_str())
        {
            return Err(AppError::invalid("AI connection settings are invalid"));
        }
        crate::ai::transport::validate_endpoint(&connection.base_url)?;
        let provider = connection.provider;
        let descriptor = crate::ai::providers::registry::descriptor(provider)
            .ok_or_else(|| AppError::unsupported("AI provider is not configurable"))?;
        for model in &connection.models {
            if let Some(profile) = &model.builtin_agent {
                if !model
                    .capabilities
                    .contains(&crate::ai::contracts::ExecutionCapability::TextGeneration)
                {
                    return Err(AppError::invalid(
                        "Builtin Agent models must support text generation",
                    ));
                }
                super::builtin_agent::validate(provider, profile)?;
            }
        }
        let mut model_ids = HashSet::new();
        let mut provider_model_ids = HashSet::new();
        if connection.models.iter().any(|model| {
            model.id.trim().is_empty()
                || model.name.trim().is_empty()
                || model.model.trim().is_empty()
                || model.capabilities.is_empty()
                || !model_ids.insert(model.id.as_str())
                || !provider_model_ids.insert(model.model.as_str())
                || model.capabilities.iter().collect::<HashSet<_>>().len()
                    != model.capabilities.len()
                || model
                    .capabilities
                    .iter()
                    .any(|capability| !descriptor.capabilities.contains(capability))
        }) {
            return Err(AppError::invalid("AI model settings are invalid"));
        }
    }
    let mut agent_kinds = HashSet::new();
    if settings
        .desktop_agents
        .iter()
        .any(|agent| !agent.enabled || !agent_kinds.insert(agent.kind))
    {
        return Err(AppError::invalid("Desktop Agent settings are invalid"));
    }
    validate_defaults(settings)
}

fn validate_defaults(settings: &AiSettings) -> AppResult<()> {
    validate_default(settings, settings.defaults.default_target.as_ref(), None)?;
    validate_default(
        settings,
        settings.defaults.writing.as_ref(),
        Some(RoutingPurpose::Writing),
    )?;
    validate_default(
        settings,
        settings.defaults.knowledge.as_ref(),
        Some(RoutingPurpose::Knowledge),
    )?;
    validate_default(
        settings,
        settings.defaults.translation.as_ref(),
        Some(RoutingPurpose::Translation),
    )?;
    validate_default(
        settings,
        settings.defaults.visual_understanding.as_ref(),
        Some(RoutingPurpose::VisualUnderstanding),
    )?;
    validate_default(
        settings,
        settings.defaults.image_generation.as_ref(),
        Some(RoutingPurpose::ImageGeneration),
    )?;
    validate_default(
        settings,
        settings.defaults.video_generation.as_ref(),
        Some(RoutingPurpose::VideoGeneration),
    )
}

fn validate_default(
    settings: &AiSettings,
    target: Option<&TaskTargetRef>,
    purpose: Option<RoutingPurpose>,
) -> AppResult<()> {
    let Some(target) = target else {
        return Ok(());
    };
    if let TaskTargetRef::Agent { runtime_kind } = target {
        return super::ensure_enabled(settings, *runtime_kind);
    }
    let TaskTargetRef::Model {
        connection_id,
        model_id,
    } = target
    else {
        unreachable!()
    };
    let model = settings
        .connections
        .iter()
        .find(|connection| connection.id == *connection_id)
        .and_then(|connection| connection.models.iter().find(|model| model.id == *model_id));
    if model.is_none_or(|model| {
        purpose.is_some_and(|purpose| !model.capabilities.contains(&purpose.execution_capability()))
    }) {
        return Err(AppError::invalid("AI default references an invalid model"));
    }
    Ok(())
}
