mod video;

pub use video::DoubaoVideoAdapter;

use crate::ai::{
    contracts::{AiProvider, ImageGenerationCapability, VideoGenerationCapability},
    providers::openai::OpenAiImageAdapter,
};

pub struct DoubaoProvider {
    image: OpenAiImageAdapter,
    video: DoubaoVideoAdapter,
}

impl DoubaoProvider {
    pub fn new() -> Self {
        Self {
            image: OpenAiImageAdapter,
            video: DoubaoVideoAdapter,
        }
    }
}

impl Default for DoubaoProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl AiProvider for DoubaoProvider {
    fn as_image_generation(&self) -> Option<&dyn ImageGenerationCapability> {
        Some(&self.image)
    }

    fn as_video_generation(&self) -> Option<&dyn VideoGenerationCapability> {
        Some(&self.video)
    }
}
