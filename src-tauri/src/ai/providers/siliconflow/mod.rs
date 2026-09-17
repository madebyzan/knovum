mod video;

pub use video::SiliconFlowVideoAdapter;

use genai::adapter::AdapterKind;

use crate::ai::{
    contracts::{AiProvider, ChatCapability, ImageGenerationCapability, VideoGenerationCapability},
    providers::openai::{OpenAiChatAdapter, OpenAiImageAdapter},
};

pub struct SiliconFlowProvider {
    chat: OpenAiChatAdapter,
    image: OpenAiImageAdapter,
    video: SiliconFlowVideoAdapter,
}

impl SiliconFlowProvider {
    pub fn new() -> Self {
        Self {
            chat: OpenAiChatAdapter::new(AdapterKind::OpenAI),
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
