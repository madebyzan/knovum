mod video;

pub use video::OpenRouterVideoAdapter;

use crate::ai::{
    contracts::{AiProvider, ImageGenerationCapability, VideoGenerationCapability},
    providers::openai::OpenAiImageAdapter,
};

pub struct OpenRouterProvider {
    image: OpenAiImageAdapter,
    video: OpenRouterVideoAdapter,
}

impl OpenRouterProvider {
    pub fn new() -> Self {
        Self {
            image: OpenAiImageAdapter,
            video: OpenRouterVideoAdapter,
        }
    }
}

impl Default for OpenRouterProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl AiProvider for OpenRouterProvider {
    fn as_image_generation(&self) -> Option<&dyn ImageGenerationCapability> {
        Some(&self.image)
    }

    fn as_video_generation(&self) -> Option<&dyn VideoGenerationCapability> {
        Some(&self.video)
    }
}
