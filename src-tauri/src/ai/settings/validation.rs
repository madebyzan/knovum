use std::collections::HashSet;

use crate::ai::{
    contracts::RoutingPurpose,
    error::{AiError, AiResult},
    models::ExecutionTargetRef,
};

use super::contracts::AiSettings;

pub(super) fn validate(settings: &AiSettings) -> AiResult<()> {
    let mut connection_ids = HashSet::new();
    for connection in &settings.connections {
        if connection.id.trim().is_empty()
            || connection.name.trim().is_empty()
            || connection.base_url.trim().is_empty()
            || !connection_ids.insert(connection.id.as_str())
        {
            return Err(AiError::invalid("AI connection settings are invalid"));
        }
        let endpoint = reqwest::Url::parse(&connection.base_url)
            .map_err(|_| AiError::invalid("AI connection Base URL is invalid"))?;
        if !matches!(endpoint.scheme(), "http" | "https") {
            return Err(AiError::invalid(
                "AI connection Base URL must use HTTP or HTTPS",
            ));
        }
        let provider = connection.provider;
        let descriptor = crate::ai::providers::registry::descriptor(provider)
            .ok_or_else(|| AiError::unsupported("AI provider is not configurable"))?;
        let mut model_ids = HashSet::new();
        let mut provider_model_ids = HashSet::new();
        if connection.models.is_empty()
            || connection.models.iter().any(|model| {
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
            })
        {
            return Err(AiError::invalid("AI model settings are invalid"));
        }
    }
    let mut agent_kinds = HashSet::new();
    if settings
        .desktop_agents
        .iter()
        .any(|agent| !agent.enabled || !agent_kinds.insert(agent.kind))
    {
        return Err(AiError::invalid("Desktop Agent settings are invalid"));
    }
    validate_defaults(settings)
}

fn validate_defaults(settings: &AiSettings) -> AiResult<()> {
    validate_default(
        settings,
        settings.defaults.writing.as_ref(),
        RoutingPurpose::Writing,
    )?;
    validate_default(
        settings,
        settings.defaults.knowledge.as_ref(),
        RoutingPurpose::Knowledge,
    )?;
    validate_default(
        settings,
        settings.defaults.translation.as_ref(),
        RoutingPurpose::Translation,
    )?;
    validate_default(
        settings,
        settings.defaults.visual_understanding.as_ref(),
        RoutingPurpose::VisualUnderstanding,
    )?;
    validate_default(
        settings,
        settings.defaults.image_generation.as_ref(),
        RoutingPurpose::ImageGeneration,
    )?;
    validate_default(
        settings,
        settings.defaults.video_generation.as_ref(),
        RoutingPurpose::VideoGeneration,
    )
}

fn validate_default(
    settings: &AiSettings,
    target: Option<&ExecutionTargetRef>,
    purpose: RoutingPurpose,
) -> AiResult<()> {
    let Some(target) = target else {
        return Ok(());
    };
    let capability = purpose.execution_capability();
    let ExecutionTargetRef::Model {
        connection_id,
        model_id,
    } = target;
    let model = settings
        .connections
        .iter()
        .find(|connection| connection.id == *connection_id)
        .and_then(|connection| connection.models.iter().find(|model| model.id == *model_id));
    if model.is_none_or(|model| !model.capabilities.contains(&capability)) {
        return Err(AiError::invalid("AI default references an invalid model"));
    }
    Ok(())
}
