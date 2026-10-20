use super::{
    contracts::{
        ImageExecutionTarget, ImageOperationBlockReason as Reason, ImageOperationCapabilities,
        ImageOperationCapability as Capability, ImageOperationTargetOption,
    },
    routing,
};
use crate::{
    ai::{
        contracts::{ExecutionCapability, RoutingPurpose},
        settings,
        tasks::{TaskExecutionTarget, TaskTarget, TaskTargetSelection},
    },
    local_resources::{
        model_catalog::{LocalModelCapability as C, MODELS},
        model_selection,
    },
};
use tauri::AppHandle;

pub async fn capabilities(app: &AppHandle) -> ImageOperationCapabilities {
    let mut targets = Vec::new();
    for model in MODELS.iter().filter(|model| {
        matches!(
            model.capability(),
            C::ImageUpscaling
                | C::ImageInpainting
                | C::ImageSelection
                | C::ImageDepth
                | C::ImageSegmentation
        )
    }) {
        let selected = model_selection::is_selected(app, *model);
        let spec_id = model
            .image_spec()
            .expect("Image capability models require a complete snapshot");
        targets.push(ImageOperationTargetOption {
            execution: ImageExecutionTarget::Local {
                model: *model,
                spec_id,
            },
            label: model.name().into(),
            capability: model.capability(),
            availability: if selected.is_ok() {
                Capability::Ready
            } else {
                blocked(Reason::ConfigurationIncomplete)
            },
            selected: selected.unwrap_or(false),
            maximum_side: if model.capability() == C::ImageUpscaling {
                Some(4096)
            } else {
                None
            },
            maximum_pixels: if model.capability() == C::ImageUpscaling {
                Some(4_194_304)
            } else {
                None
            },
        });
    }
    let mut capabilities = ImageOperationCapabilities {
        remote: None,
        upscale: selected_availability(&targets, C::ImageUpscaling),
        multi_angle: blocked(Reason::ConfigurationIncomplete),
        masked_removal: selected_availability(&targets, C::ImageInpainting),
        prompt_removal: blocked(Reason::ConfigurationIncomplete),
        selection: selected_availability(&targets, C::ImageSelection),
        depth: selected_availability(&targets, C::ImageDepth),
        targets,
    };
    let Ok(selection) = settings::default_selection(app, RoutingPurpose::ImageGeneration) else {
        return capabilities;
    };
    let reference = match &selection {
        TaskTargetSelection::Model {
            connection_id,
            model_id,
            ..
        } => TaskTarget::Model {
            connection_id: connection_id.clone(),
            model_id: model_id.clone(),
        },
        TaskTargetSelection::Agent { runtime_kind } => TaskTarget::Agent {
            runtime_kind: *runtime_kind,
        },
    };
    let Ok(target) = settings::resolve_task(app, selection, ExecutionCapability::ImageGeneration)
    else {
        return capabilities;
    };
    match &target {
        TaskExecutionTarget::Agent { .. } => {
            capabilities.multi_angle = Capability::Unverified;
            capabilities.prompt_removal = Capability::Unverified;
        }
        TaskExecutionTarget::Model(model) => {
            let available = match crate::ai::transport::SecurityContext::from_target(model).await {
                Ok(_) => Capability::Ready,
                Err(error) if error.code == "not_configured" => {
                    blocked(Reason::ConfigurationIncomplete)
                }
                Err(_) => blocked(Reason::CredentialUnavailable),
            };
            let supported = if routing::supports_remote_operations(model) {
                available
            } else {
                blocked(Reason::ProviderUnsupported)
            };
            capabilities.multi_angle = supported;
            capabilities.prompt_removal = supported;
        }
    }
    capabilities.remote = Some(ImageExecutionTarget::Remote { target: reference });
    capabilities
}

fn blocked(reason: Reason) -> Capability {
    Capability::Blocked { reason }
}

fn selected_availability(targets: &[ImageOperationTargetOption], capability: C) -> Capability {
    targets
        .iter()
        .find(|option| option.capability == capability && option.selected)
        .map(|option| option.availability)
        .unwrap_or_else(|| blocked(Reason::ConfigurationIncomplete))
}
