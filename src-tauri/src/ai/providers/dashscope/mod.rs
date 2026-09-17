mod chat;
mod image;
mod protocol;
mod upload;
mod video;

#[cfg(test)]
mod tests;

pub use chat::DashScopeChatAdapter;
pub use image::DashScopeImageAdapter;
pub use upload::DashScopeFileUploadAdapter;
pub use video::DashScopeVideoAdapter;

use crate::ai::contracts::{
    AiProvider, ChatCapability, FileUploadCapability, ImageGenerationCapability,
    VideoGenerationCapability,
};

pub struct DashScopeProvider {
    chat: DashScopeChatAdapter,
    image: DashScopeImageAdapter,
    video: DashScopeVideoAdapter,
    upload: DashScopeFileUploadAdapter,
}

impl DashScopeProvider {
    pub fn new() -> Self {
        Self {
            chat: DashScopeChatAdapter,
            image: DashScopeImageAdapter,
            video: DashScopeVideoAdapter,
            upload: DashScopeFileUploadAdapter,
        }
    }
}

impl Default for DashScopeProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl AiProvider for DashScopeProvider {
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
