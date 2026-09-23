mod upload;
mod video;

pub use upload::ZhipuFileUploadAdapter;
pub use video::ZhipuVideoAdapter;

use crate::ai::{
    contracts::{
        AiProvider, FileUploadCapability, ImageGenerationCapability, VideoGenerationCapability,
    },
    providers::openai::OpenAiImageAdapter,
};

pub struct ZhipuProvider {
    image: OpenAiImageAdapter,
    video: ZhipuVideoAdapter,
    upload: ZhipuFileUploadAdapter,
}

impl ZhipuProvider {
    pub fn new() -> Self {
        Self {
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
