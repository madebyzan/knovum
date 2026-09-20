use genai::adapter::AdapterKind;

use crate::ai::{
    contracts::{AiProvider, ChatCapability, ImageGenerationCapability},
    providers::openai::{OpenAiChatAdapter, OpenAiImageAdapter},
};

pub struct GrokProvider {
    chat: OpenAiChatAdapter,
    image: OpenAiImageAdapter,
}

impl GrokProvider {
    pub fn new() -> Self {
        Self {
            chat: OpenAiChatAdapter::new(AdapterKind::OpenAI),
            image: OpenAiImageAdapter,
        }
    }
}

impl Default for GrokProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl AiProvider for GrokProvider {
    fn as_chat(&self) -> Option<&dyn ChatCapability> {
        Some(&self.chat)
    }

    fn as_image_generation(&self) -> Option<&dyn ImageGenerationCapability> {
        Some(&self.image)
    }
}
