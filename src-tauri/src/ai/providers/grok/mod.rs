mod upload;
mod video;

pub use upload::GrokFileUploadAdapter;
pub use video::GrokVideoAdapter;

use crate::ai::{
    contracts::{
        AiProvider, FileUploadCapability, ImageGenerationCapability, VideoGenerationCapability,
    },
    providers::openai::OpenAiImageAdapter,
};

pub struct GrokProvider {
    image: OpenAiImageAdapter,
    video: GrokVideoAdapter,
    upload: GrokFileUploadAdapter,
}

impl GrokProvider {
    pub fn new() -> Self {
        Self {
            image: OpenAiImageAdapter,
            video: GrokVideoAdapter,
            upload: GrokFileUploadAdapter,
        }
    }
}

impl Default for GrokProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl AiProvider for GrokProvider {
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
