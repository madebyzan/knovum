mod chat;
mod image;
mod upload;

pub use chat::OpenAiChatAdapter;
pub use image::OpenAiImageAdapter;
pub use upload::OpenAiFileUploadAdapter;

use genai::adapter::AdapterKind;

use crate::ai::contracts::{
    AiProvider, ChatCapability, FileUploadCapability, ImageGenerationCapability, ProviderKind,
};

pub struct OpenAiProvider {
    kind: ProviderKind,
    chat: OpenAiChatAdapter,
    image: OpenAiImageAdapter,
    upload: OpenAiFileUploadAdapter,
}

impl OpenAiProvider {
    pub fn new(kind: ProviderKind) -> Self {
        let adapter = match kind {
            ProviderKind::DeepSeek => AdapterKind::DeepSeek,
            ProviderKind::Groq => AdapterKind::Groq,
            _ => AdapterKind::OpenAI,
        };
        Self {
            kind,
            chat: OpenAiChatAdapter::new(adapter),
            image: OpenAiImageAdapter,
            upload: OpenAiFileUploadAdapter,
        }
    }
}

impl AiProvider for OpenAiProvider {
    fn as_chat(&self) -> Option<&dyn ChatCapability> {
        Some(&self.chat)
    }

    fn as_image_generation(&self) -> Option<&dyn ImageGenerationCapability> {
        Some(&self.image)
    }

    fn as_file_upload(&self) -> Option<&dyn FileUploadCapability> {
        if self.kind == ProviderKind::OpenAi {
            Some(&self.upload)
        } else {
            None
        }
    }
}
