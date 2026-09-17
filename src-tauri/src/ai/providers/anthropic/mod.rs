use genai::adapter::AdapterKind;

use crate::ai::{
    contracts::{AiProvider, ChatCapability},
    providers::openai::OpenAiChatAdapter,
};

pub struct AnthropicProvider {
    chat: OpenAiChatAdapter,
}

impl AnthropicProvider {
    pub fn new() -> Self {
        Self {
            chat: OpenAiChatAdapter::new(AdapterKind::Anthropic),
        }
    }
}

impl Default for AnthropicProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl AiProvider for AnthropicProvider {
    fn as_chat(&self) -> Option<&dyn ChatCapability> {
        Some(&self.chat)
    }
}
