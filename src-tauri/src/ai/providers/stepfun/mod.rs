mod video;

pub use video::StepFunVideoAdapter;

use genai::adapter::AdapterKind;

use crate::ai::{
    contracts::{AiProvider, ChatCapability, ImageGenerationCapability, VideoGenerationCapability},
    providers::openai::{OpenAiChatAdapter, OpenAiImageAdapter},
};

pub struct StepFunProvider {
    chat: OpenAiChatAdapter,
    image: OpenAiImageAdapter,
    video: StepFunVideoAdapter,
}

impl StepFunProvider {
    pub fn new() -> Self {
        Self {
            chat: OpenAiChatAdapter::new(AdapterKind::OpenAI),
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
    fn as_chat(&self) -> Option<&dyn ChatCapability> {
        Some(&self.chat)
    }

    fn as_image_generation(&self) -> Option<&dyn ImageGenerationCapability> {
        Some(&self.image)
    }

    fn as_video_generation(&self) -> Option<&dyn VideoGenerationCapability> {
        Some(&self.video)
    }
}
