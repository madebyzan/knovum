mod image;
mod video;

pub use image::MiniMaxImageAdapter;
pub use video::MiniMaxVideoAdapter;

use genai::adapter::AdapterKind;

use crate::ai::{
    contracts::{AiProvider, ChatCapability, ImageGenerationCapability, VideoGenerationCapability},
    providers::openai::OpenAiChatAdapter,
};

pub struct MiniMaxProvider {
    chat: OpenAiChatAdapter,
    image: MiniMaxImageAdapter,
    video: MiniMaxVideoAdapter,
}

impl MiniMaxProvider {
    pub fn new() -> Self {
        Self {
            chat: OpenAiChatAdapter::new(AdapterKind::OpenAI),
            image: MiniMaxImageAdapter,
            video: MiniMaxVideoAdapter,
        }
    }
}

impl Default for MiniMaxProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl AiProvider for MiniMaxProvider {
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
