mod image;
mod image_operation;
mod image_operation_protocol;
mod protocol;
mod video;

#[cfg(test)]
mod tests;

pub use image::DashScopeImageAdapter;
pub use video::DashScopeVideoAdapter;

use crate::ai::contracts::{AiProvider, ImageGenerationCapability, VideoGenerationCapability};

pub struct DashScopeProvider {
    image: DashScopeImageAdapter,
    video: DashScopeVideoAdapter,
}

impl DashScopeProvider {
    pub fn new() -> Self {
        Self {
            image: DashScopeImageAdapter,
            video: DashScopeVideoAdapter,
        }
    }
}

impl Default for DashScopeProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl AiProvider for DashScopeProvider {
    fn as_image_generation(&self) -> Option<&dyn ImageGenerationCapability> {
        Some(&self.image)
    }

    fn as_video_generation(&self) -> Option<&dyn VideoGenerationCapability> {
        Some(&self.video)
    }
}

fn endpoint(value: &str) -> String {
    let value = value.trim().trim_end_matches('/');
    let value = value.strip_suffix("/compatible-mode/v1").unwrap_or(value);
    let value = value.strip_suffix("/api/v1").unwrap_or(value);
    format!("{value}/api/v1")
}
