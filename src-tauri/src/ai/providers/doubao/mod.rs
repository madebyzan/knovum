mod video;

pub use video::DoubaoVideoAdapter;

use genai::adapter::AdapterKind;

use crate::ai::{
    contracts::{AiProvider, ChatCapability, ImageGenerationCapability, VideoGenerationCapability},
    providers::openai::{OpenAiChatAdapter, OpenAiImageAdapter},
};

pub struct DoubaoProvider {
    chat: OpenAiChatAdapter,
    image: OpenAiImageAdapter,
    video: DoubaoVideoAdapter,
}

impl DoubaoProvider {
    pub fn new() -> Self {
        Self {
            chat: OpenAiChatAdapter::new(AdapterKind::OpenAI),
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
