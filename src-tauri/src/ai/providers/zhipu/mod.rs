mod video;

pub use video::ZhipuVideoAdapter;

use crate::ai::{
    contracts::{AiProvider, ImageGenerationCapability, VideoGenerationCapability},
    providers::openai::OpenAiImageAdapter,
};

pub struct ZhipuProvider {
    image: OpenAiImageAdapter,
    video: ZhipuVideoAdapter,
}

impl ZhipuProvider {
    pub fn new() -> Self {
        Self {
            image: OpenAiImageAdapter,
            video: ZhipuVideoAdapter,
        }
    }
}

impl Default for ZhipuProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl AiProvider for ZhipuProvider {
    fn as_image_generation(&self) -> Option<&dyn ImageGenerationCapability> {
        Some(&self.image)
    }

    fn as_video_generation(&self) -> Option<&dyn VideoGenerationCapability> {
        Some(&self.video)
    }
}
