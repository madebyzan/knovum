mod video;

pub use video::SiliconFlowVideoAdapter;

use crate::ai::{
    contracts::{AiProvider, ImageGenerationCapability, VideoGenerationCapability},
    providers::openai::OpenAiImageAdapter,
};

pub struct SiliconFlowProvider {
    image: OpenAiImageAdapter,
    video: SiliconFlowVideoAdapter,
}

impl SiliconFlowProvider {
    pub fn new() -> Self {
        Self {
            image: OpenAiImageAdapter,
            video: SiliconFlowVideoAdapter,
        }
    }
}

impl Default for SiliconFlowProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl AiProvider for SiliconFlowProvider {
    fn as_image_generation(&self) -> Option<&dyn ImageGenerationCapability> {
        Some(&self.image)
    }

    fn as_video_generation(&self) -> Option<&dyn VideoGenerationCapability> {
        Some(&self.video)
    }
}
