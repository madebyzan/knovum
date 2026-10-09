mod image;
mod image_edit;
mod upload;
mod video;

#[cfg(test)]
#[path = "tests/video.rs"]
mod video_tests;

pub use image::GeminiImageAdapter;
pub use image_edit::edit_image;
pub use upload::GeminiFileUploadAdapter;
pub use video::GeminiVideoAdapter;

use crate::ai::contracts::{
    AiProvider, FileUploadCapability, ImageGenerationCapability, VideoGenerationCapability,
};

pub struct GeminiProvider {
    image: GeminiImageAdapter,
    video: GeminiVideoAdapter,
    upload: GeminiFileUploadAdapter,
}

impl GeminiProvider {
    pub fn new() -> Self {
        Self {
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

#[cfg(test)]
#[path = "tests/upload.rs"]
mod upload_tests;
