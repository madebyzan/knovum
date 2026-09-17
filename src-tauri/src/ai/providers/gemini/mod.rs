mod chat;
mod image;
mod upload;
mod video;

pub use chat::GeminiChatAdapter;
pub use image::GeminiImageAdapter;
pub use upload::GeminiFileUploadAdapter;
pub use video::GeminiVideoAdapter;

use crate::ai::contracts::{
    AiProvider, ChatCapability, FileUploadCapability, ImageGenerationCapability,
    VideoGenerationCapability,
};

pub struct GeminiProvider {
    chat: GeminiChatAdapter,
    image: GeminiImageAdapter,
    video: GeminiVideoAdapter,
    upload: GeminiFileUploadAdapter,
}

impl GeminiProvider {
    pub fn new() -> Self {
        Self {
            chat: GeminiChatAdapter,
            image: GeminiImageAdapter,
            video: GeminiVideoAdapter,
            upload: GeminiFileUploadAdapter,
        }
    }
}

impl Default for GeminiProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl AiProvider for GeminiProvider {
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
