mod image;
mod image_edit;
mod image_task;
mod video;

pub use image::KlingImageAdapter;
pub use image_edit::edit_source;
pub use video::KlingVideoAdapter;

use crate::ai::contracts::{AiProvider, ImageGenerationCapability, VideoGenerationCapability};

pub struct KlingProvider {
    image: KlingImageAdapter,
    video: KlingVideoAdapter,
}

impl KlingProvider {
    pub fn new() -> Self {
        Self {
            image: KlingImageAdapter,
            video: KlingVideoAdapter,
        }
    }
}

impl Default for KlingProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl AiProvider for KlingProvider {
    fn as_image_generation(&self) -> Option<&dyn ImageGenerationCapability> {
        Some(&self.image)
    }

    fn as_video_generation(&self) -> Option<&dyn VideoGenerationCapability> {
        Some(&self.video)
    }
}
