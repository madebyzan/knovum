mod video;

pub use video::StepFunVideoAdapter;

use crate::ai::{
    contracts::{AiProvider, ImageGenerationCapability, VideoGenerationCapability},
    providers::openai::OpenAiImageAdapter,
};

pub struct StepFunProvider {
    image: OpenAiImageAdapter,
    video: StepFunVideoAdapter,
}

impl StepFunProvider {
    pub fn new() -> Self {
        Self {
            image: OpenAiImageAdapter,
            video: StepFunVideoAdapter,
        }
    }
}

impl Default for StepFunProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl AiProvider for StepFunProvider {
    fn as_image_generation(&self) -> Option<&dyn ImageGenerationCapability> {
        Some(&self.image)
    }

    fn as_video_generation(&self) -> Option<&dyn VideoGenerationCapability> {
        Some(&self.video)
    }
}
