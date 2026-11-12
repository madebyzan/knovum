mod image;
mod upload;
mod video;

#[cfg(test)]
mod tests;

pub use image::GeminiImageAdapter;
pub use video::GeminiVideoAdapter;

use crate::ai::contracts::{AiProvider, ImageGenerationCapability, VideoGenerationCapability};

pub struct GeminiProvider {
    image: GeminiImageAdapter,
    video: GeminiVideoAdapter,
}

impl GeminiProvider {
    pub fn new() -> Self {
        Self {
            image: GeminiImageAdapter,
            video: GeminiVideoAdapter,
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
}
