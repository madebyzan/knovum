use crate::ai::{
    contracts::{AiProvider, ImageGenerationCapability},
    providers::openai::OpenAiImageAdapter,
};

pub struct GrokProvider {
    image: OpenAiImageAdapter,
}

impl GrokProvider {
    pub fn new() -> Self {
        Self {
            image: OpenAiImageAdapter,
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
}
