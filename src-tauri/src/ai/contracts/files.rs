use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};

use crate::ai::transport::SecurityContext;
use crate::foundation::app_error::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadedFileReference {
    pub file_id: String,
    pub access_url: Option<String>,
    pub provider: String,
}

pub trait FileUploadCapability: Send + Sync {
    fn upload_ephemeral_file<'a>(
        &'a self,
        ctx: &'a SecurityContext,
        bytes: Vec<u8>,
        mime_type: &'a str,
        file_name: &'a str,
    ) -> BoxFuture<'a, AppResult<UploadedFileReference>>;
}
