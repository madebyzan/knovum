mod image;

pub use image::OpenAiImageAdapter;

use crate::ai::contracts::{
    AiProvider, ExecutionCapability, ImageGenerationCapability, ProviderKind,
};
use crate::ai::providers::registry;

pub struct OpenAiProvider {
    kind: ProviderKind,
    image: OpenAiImageAdapter,
}

impl OpenAiProvider {
    pub fn new(kind: ProviderKind) -> Self {
        Self {
            kind,
            image: OpenAiImageAdapter,
        }
    }
}

impl AiProvider for OpenAiProvider {
    fn as_image_generation(&self) -> Option<&dyn ImageGenerationCapability> {
        // This adapter serves every OpenAI-compatible vendor, but only the
        // ones whose descriptor declares image generation actually offer it.
        let declares_image = registry::descriptor(self.kind).is_some_and(|descriptor| {
            descriptor
                .capabilities
                .contains(&ExecutionCapability::ImageGeneration)
        });
        declares_image.then_some(&self.image as &dyn ImageGenerationCapability)
    }
}
