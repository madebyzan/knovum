mod video;

pub use video::TogetherVideoAdapter;

use crate::ai::{
    contracts::{AiProvider, ImageGenerationCapability, VideoGenerationCapability},
    providers::openai::OpenAiImageAdapter,
};

pub struct TogetherProvider {
    image: OpenAiImageAdapter,
    video: TogetherVideoAdapter,
}

impl TogetherProvider {
    pub fn new() -> Self {
        Self {
            image: OpenAiImageAdapter,
            video: TogetherVideoAdapter,
        }
    }
}

impl Default for TogetherProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl AiProvider for TogetherProvider {
    fn as_image_generation(&self) -> Option<&dyn ImageGenerationCapability> {
        Some(&self.image)
    }

    fn as_video_generation(&self) -> Option<&dyn VideoGenerationCapability> {
        Some(&self.video)
    }
}
