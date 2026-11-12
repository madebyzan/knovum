mod image;
mod video;

pub use image::HunyuanImageAdapter;
pub use video::HunyuanVideoAdapter;

use crate::ai::contracts::{AiProvider, ImageGenerationCapability, VideoGenerationCapability};

pub struct HunyuanProvider {
    image: HunyuanImageAdapter,
    video: HunyuanVideoAdapter,
}

impl HunyuanProvider {
    pub fn new() -> Self {
        Self {
            image: HunyuanImageAdapter,
            video: HunyuanVideoAdapter,
        }
    }
}

impl Default for HunyuanProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl AiProvider for HunyuanProvider {
    fn as_image_generation(&self) -> Option<&dyn ImageGenerationCapability> {
        Some(&self.image)
    }

    fn as_video_generation(&self) -> Option<&dyn VideoGenerationCapability> {
        Some(&self.video)
    }
}

/// TokenHub serves chat, image and video from one host under `/v1`; the media
/// paths are appended below it.
fn endpoint(raw: &str) -> String {
    let trimmed = raw.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        "https://tokenhub.tencentmaas.com/v1".to_string()
    } else if trimmed.ends_with("/v1") {
        trimmed.to_string()
    } else {
        format!("{trimmed}/v1")
    }
}
