mod image;
mod video;

pub use image::OpenRouterImageAdapter;
pub use video::OpenRouterVideoAdapter;

use crate::ai::contracts::{AiProvider, ImageGenerationCapability, VideoGenerationCapability};

pub struct OpenRouterProvider {
    image: OpenRouterImageAdapter,
    video: OpenRouterVideoAdapter,
}

impl OpenRouterProvider {
    pub fn new() -> Self {
        Self {
            image: OpenRouterImageAdapter,
            video: OpenRouterVideoAdapter,
        }
    }
}

impl AiProvider for OpenRouterProvider {
    fn as_image_generation(&self) -> Option<&dyn ImageGenerationCapability> {
        Some(&self.image)
    }

    fn as_video_generation(&self) -> Option<&dyn VideoGenerationCapability> {
        Some(&self.video)
    }
}

/// OpenRouter bases arrive with or without `/api/v1`.
pub(super) fn endpoint(raw: &str) -> String {
    let trimmed = raw.trim().trim_end_matches('/');
    if trimmed.ends_with("/v1") {
        trimmed.to_string()
    } else {
        format!("{trimmed}/api/v1")
    }
}
