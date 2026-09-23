mod image;
mod upload;

pub use image::OpenAiImageAdapter;
pub use upload::OpenAiFileUploadAdapter;

use crate::ai::contracts::{
    AiProvider, FileUploadCapability, ImageGenerationCapability, ProviderKind,
};

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
        Some(&self.image)
    }

    fn as_file_upload(&self) -> Option<&dyn FileUploadCapability> {
        if self.kind == ProviderKind::OpenAi {
            Some(&self.upload)
        } else {
            None
        }
    }
}
