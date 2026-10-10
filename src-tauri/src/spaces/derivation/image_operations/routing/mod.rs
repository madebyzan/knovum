mod target;
use crate::ai::{
    contracts::ProviderKind, models::ModelExecutionTarget, providers::image_edit::ImageEditApi,
};
pub(super) use target::local_capability;
pub(super) use target::{resolve, ResolvedImageTarget};

use super::contracts::ImageOperation;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ImageOperationBackend {
    DashScope,
    Gemini,
    Kling,
    ImageEdit(ImageEditApi),
    Unsupported,
}

pub(super) struct SupportedOperations {
    pub multi_angle: bool,
    pub prompt_removal: bool,
}

impl SupportedOperations {
    pub fn allows(&self, operation: &ImageOperation) -> bool {
        match operation {
            ImageOperation::MultiAngle { .. } => self.multi_angle,
            ImageOperation::RemoveObjectPrompt { .. } => self.prompt_removal,
            _ => false,
        }
    }
}

pub(super) fn supported_operations(backend: ImageOperationBackend) -> SupportedOperations {
    SupportedOperations {
        multi_angle: backend == ImageOperationBackend::DashScope,
        prompt_removal: matches!(
            backend,
            ImageOperationBackend::Gemini
                | ImageOperationBackend::Kling
                | ImageOperationBackend::ImageEdit(_)
        ),
    }
}

pub(super) fn backend(target: &ModelExecutionTarget) -> ImageOperationBackend {
    match (ProviderKind::parse(target.provider()), target.model()) {
        (Some(ProviderKind::DashScope), _) => ImageOperationBackend::DashScope,
        (Some(ProviderKind::Gemini), model)
            if model.starts_with("gemini-") && model.contains("-image") =>
        {
            ImageOperationBackend::Gemini
        }
        (Some(ProviderKind::Grok), "grok-imagine-image-2.0") => {
            ImageOperationBackend::ImageEdit(ImageEditApi::Grok)
        }
        (Some(ProviderKind::Doubao), model) if model.starts_with("doubao-seedream-") => {
            ImageOperationBackend::ImageEdit(ImageEditApi::Doubao)
        }
        (Some(ProviderKind::OpenRouter), model) if supports_openrouter_edit(model) => {
            ImageOperationBackend::ImageEdit(ImageEditApi::OpenRouter)
        }
        (Some(ProviderKind::SiliconFlow), model) if model.starts_with("Qwen/Qwen-Image-Edit") => {
            ImageOperationBackend::ImageEdit(ImageEditApi::SiliconFlow)
        }
        (Some(ProviderKind::Kling), "kling-image-o1" | "kling-v3-omni") => {
            ImageOperationBackend::Kling
        }
        _ => ImageOperationBackend::Unsupported,
    }
}

fn supports_openrouter_edit(model: &str) -> bool {
    model.starts_with("openai/gpt-image-")
        || model.starts_with("google/gemini-") && model.contains("-image")
        || model.starts_with("bytedance-seed/seedream-")
        || matches!(
            model,
            "black-forest-labs/flux.2-pro"
                | "black-forest-labs/flux.2-max"
                | "black-forest-labs/flux.2-flex"
                | "black-forest-labs/flux.2-klein-4b"
        )
}

#[cfg(test)]
mod tests;
