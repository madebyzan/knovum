mod upload;

pub use upload::MoonshotFileUploadAdapter;

use genai::adapter::AdapterKind;

use crate::ai::{
    contracts::{AiProvider, ChatCapability, FileUploadCapability},
    providers::openai::OpenAiChatAdapter,
};

pub struct MoonshotProvider {
    chat: OpenAiChatAdapter,
    upload: MoonshotFileUploadAdapter,
}

impl MoonshotProvider {
    pub fn new() -> Self {
        Self {
            chat: OpenAiChatAdapter::new(AdapterKind::OpenAI),
            upload: MoonshotFileUploadAdapter,
        }
    }
}

impl Default for MoonshotProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl AiProvider for MoonshotProvider {
    fn as_chat(&self) -> Option<&dyn ChatCapability> {
        Some(&self.chat)
    }

    fn as_file_upload(&self) -> Option<&dyn FileUploadCapability> {
        Some(&self.upload)
    }
}
