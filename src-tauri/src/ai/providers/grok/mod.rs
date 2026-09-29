mod upload;

pub use upload::GrokFileUploadAdapter;

use crate::ai::{
    contracts::{AiProvider, FileUploadCapability, ImageGenerationCapability},
    providers::openai::OpenAiImageAdapter,
};

pub struct GrokProvider {
    image: OpenAiImageAdapter,
    upload: GrokFileUploadAdapter,
}

impl GrokProvider {
    pub fn new() -> Self {
        Self {
            image: OpenAiImageAdapter,
            upload: GrokFileUploadAdapter,
        }
    }
}

impl Default for GrokProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl AiProvider for GrokProvider {
    fn as_image_generation(&self) -> Option<&dyn ImageGenerationCapability> {
        Some(&self.image)
    }

    fn as_file_upload(&self) -> Option<&dyn FileUploadCapability> {
        Some(&self.upload)
    }
}
