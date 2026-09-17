mod upload;
mod video;

use genai::adapter::AdapterKind;
pub use upload::ZhipuFileUploadAdapter;
pub use video::ZhipuVideoAdapter;

use crate::ai::{
    contracts::{
        AiProvider, ChatCapability, FileUploadCapability, ImageGenerationCapability,
        VideoGenerationCapability,
    },
    providers::openai::{OpenAiChatAdapter, OpenAiImageAdapter},
};

pub struct ZhipuProvider {
    chat: OpenAiChatAdapter,
    image: OpenAiImageAdapter,
    video: ZhipuVideoAdapter,
    upload: ZhipuFileUploadAdapter,
}

impl ZhipuProvider {
    pub fn new() -> Self {
        Self {
            chat: OpenAiChatAdapter::new(AdapterKind::OpenAI),
            image: OpenAiImageAdapter,
            video: ZhipuVideoAdapter,
            upload: ZhipuFileUploadAdapter,
        }
    }
}

impl Default for ZhipuProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl AiProvider for ZhipuProvider {
    fn as_chat(&self) -> Option<&dyn ChatCapability> {
        Some(&self.chat)
    }

    fn as_image_generation(&self) -> Option<&dyn ImageGenerationCapability> {
        Some(&self.image)
    }

    fn as_video_generation(&self) -> Option<&dyn VideoGenerationCapability> {
        Some(&self.video)
    }

    fn as_file_upload(&self) -> Option<&dyn FileUploadCapability> {
        Some(&self.upload)
    }
}
