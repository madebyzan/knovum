use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};

use crate::ai::{error::AiResult, transport::SecurityContext};

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
    ) -> BoxFuture<'a, AiResult<UploadedFileReference>>;
}
