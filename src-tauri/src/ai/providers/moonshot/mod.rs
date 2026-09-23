mod upload;

pub use upload::MoonshotFileUploadAdapter;

use crate::ai::contracts::{AiProvider, FileUploadCapability};

pub struct MoonshotProvider {
    upload: MoonshotFileUploadAdapter,
}

impl MoonshotProvider {
    pub fn new() -> Self {
        Self {
            upload: MoonshotFileUploadAdapter,
        }
    }
}

impl Default for MoonshotProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl AiProvider for MoonshotProvider {
    fn as_file_upload(&self) -> Option<&dyn FileUploadCapability> {
        Some(&self.upload)
    }
}
