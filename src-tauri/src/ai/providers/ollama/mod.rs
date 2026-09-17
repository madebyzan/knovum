mod chat;

pub use chat::OllamaChatAdapter;

use crate::ai::contracts::{AiProvider, ChatCapability};

pub struct OllamaProvider {
    chat: OllamaChatAdapter,
}

impl OllamaProvider {
    pub fn new() -> Self {
        Self {
            chat: OllamaChatAdapter,
        }
    }
}

impl Default for OllamaProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl AiProvider for OllamaProvider {
    fn as_chat(&self) -> Option<&dyn ChatCapability> {
        Some(&self.chat)
    }
}
