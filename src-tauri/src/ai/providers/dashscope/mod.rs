mod image;
mod image_operation;
mod image_operation_protocol;
mod protocol;
mod upload;
mod video;

#[cfg(test)]
mod tests;

pub use image::DashScopeImageAdapter;
pub(crate) use image_operation::execute_image_view;
pub use upload::DashScopeFileUploadAdapter;
pub use video::DashScopeVideoAdapter;

use crate::ai::contracts::{
    AiProvider, FileUploadCapability, ImageGenerationCapability, VideoGenerationCapability,
};

pub struct DashScopeProvider {
    image: DashScopeImageAdapter,
    video: DashScopeVideoAdapter,
    upload: DashScopeFileUploadAdapter,
}

impl DashScopeProvider {
    pub fn new() -> Self {
        Self {
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

fn endpoint(value: &str) -> String {
    let value = value.trim().trim_end_matches('/');
    let value = value.strip_suffix("/compatible-mode/v1").unwrap_or(value);
    let value = value.strip_suffix("/api/v1").unwrap_or(value);
    format!("{value}/api/v1")
}
