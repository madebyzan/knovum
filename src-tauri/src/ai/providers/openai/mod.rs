mod image;
mod upload;

pub use image::OpenAiImageAdapter;
pub use upload::OpenAiFileUploadAdapter;

use crate::ai::contracts::{
    AiProvider, ExecutionCapability, FileUploadCapability, ImageGenerationCapability, ProviderKind,
};
use crate::ai::providers::registry;

pub struct OpenAiProvider {
    kind: ProviderKind,
    image: OpenAiImageAdapter,
    upload: OpenAiFileUploadAdapter,
}

impl OpenAiProvider {
    pub fn new(kind: ProviderKind) -> Self {
        Self {
            kind,
            image: OpenAiImageAdapter,
            upload: OpenAiFileUploadAdapter,
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

    fn as_file_upload(&self) -> Option<&dyn FileUploadCapability> {
        if self.kind == ProviderKind::OpenAi {
            Some(&self.upload)
        } else {
            None
        }
    }
}
