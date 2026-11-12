mod video;

pub use video::GrokVideoAdapter;

use crate::ai::{
    contracts::{AiProvider, ImageGenerationCapability, VideoGenerationCapability},
    providers::openai::OpenAiImageAdapter,
};

pub struct GrokProvider {
    image: OpenAiImageAdapter,
    video: GrokVideoAdapter,
}

impl GrokProvider {
    pub fn new() -> Self {
        Self {
            image: OpenAiImageAdapter,
            video: GrokVideoAdapter,
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

    fn as_video_generation(&self) -> Option<&dyn VideoGenerationCapability> {
        Some(&self.video)
    }
}
