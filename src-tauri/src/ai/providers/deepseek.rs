use genai::adapter::AdapterKind;

use crate::ai::{
    contracts::{AiProvider, ChatCapability},
    providers::openai::OpenAiChatAdapter,
};

pub struct DeepSeekProvider {
    chat: OpenAiChatAdapter,
}

impl DeepSeekProvider {
    pub fn new() -> Self {
        Self {
            chat: OpenAiChatAdapter::new(AdapterKind::DeepSeek),
        }
    }
}

impl Default for DeepSeekProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl AiProvider for DeepSeekProvider {
    fn as_chat(&self) -> Option<&dyn ChatCapability> {
        Some(&self.chat)
    }
}
