use crate::ai::transport::file_upload::{upload_file, UploadFile};
use futures::future::BoxFuture;

use super::endpoint;
use crate::ai::{
    contracts::{FileUploadCapability, UploadedFileReference},
    transport::SecurityContext,
};
use crate::foundation::app_error::{AppError, AppResult};

pub struct DashScopeFileUploadAdapter;

impl FileUploadCapability for DashScopeFileUploadAdapter {
    fn upload_ephemeral_file<'a>(
        &'a self,
        ctx: &'a SecurityContext,
        bytes: Vec<u8>,
        mime_type: &'a str,
        file_name: &'a str,
    ) -> BoxFuture<'a, AppResult<UploadedFileReference>> {
        Box::pin(async move {
            let upload_url = format!("{}/files", endpoint(&ctx.endpoint));
            let file_upload_response = upload_file(
                ctx,
                &upload_url,
                UploadFile {
                    bytes,
                    mime_type,
                    file_name,
                },
                "multimodal",
            )
            .await?;

            let file_id = file_upload_response["file_id"]
                .as_str()
                .or_else(|| file_upload_response["id"].as_str())
                .ok_or_else(|| {
                    AppError::new("invalid_response", "Missing file ID in DashScope response")
                })?
                .to_string();

            let access_url = file_upload_response["url"]
                .as_str()
                .map(ToString::to_string);

            Ok(UploadedFileReference {
                file_id,
                access_url,
                provider: "dashscope".to_string(),
            })
        })
    }
}
