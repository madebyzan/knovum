mod image;
#[cfg(test)]
mod tests;
mod video;

pub use image::MiniMaxImageAdapter;
pub use video::MiniMaxVideoAdapter;

use crate::ai::contracts::{AiProvider, ImageGenerationCapability, VideoGenerationCapability};

pub struct MiniMaxProvider {
    image: MiniMaxImageAdapter,
    video: MiniMaxVideoAdapter,
}

impl MiniMaxProvider {
    pub fn new() -> Self {
        Self {
            image: MiniMaxImageAdapter,
            video: MiniMaxVideoAdapter,
        }
    }
}

impl Default for MiniMaxProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl AiProvider for MiniMaxProvider {
    fn as_image_generation(&self) -> Option<&dyn ImageGenerationCapability> {
        Some(&self.image)
    }

    fn as_video_generation(&self) -> Option<&dyn VideoGenerationCapability> {
        Some(&self.video)
    }
}

fn endpoint(raw: &str) -> String {
    let trimmed = raw.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        "https://api.minimax.chat/v1".to_string()
    } else if trimmed.ends_with("/v1") {
        trimmed.to_string()
    } else {
        format!("{trimmed}/v1")
    }
}
