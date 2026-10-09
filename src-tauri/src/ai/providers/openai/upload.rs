use crate::ai::transport::file_upload::{upload_file, UploadFile};
use futures::future::BoxFuture;

use crate::ai::{
    contracts::{FileUploadCapability, UploadedFileReference},
    transport::SecurityContext,
};
use crate::app_error::{AppError, AppResult};

pub struct OpenAiFileUploadAdapter;

impl FileUploadCapability for OpenAiFileUploadAdapter {
    fn upload_ephemeral_file<'a>(
        &'a self,
        ctx: &'a SecurityContext,
        bytes: Vec<u8>,
        mime_type: &'a str,
        file_name: &'a str,
    ) -> BoxFuture<'a, AppResult<UploadedFileReference>> {
        Box::pin(async move {
            let upload_url = format!("{}/files", openai_endpoint(&ctx.endpoint));
            let file_upload_response = upload_file(
                ctx,
                &upload_url,
                UploadFile {
                    bytes,
                    mime_type,
                    file_name,
                },
                "assistants",
            )
            .await?;

            let file_id = file_upload_response["id"]
                .as_str()
                .ok_or_else(|| {
                    AppError::new("invalid_response", "Missing file ID in OpenAI response")
                })?
                .to_string();

            Ok(UploadedFileReference {
                file_id,
                access_url: None,
                provider: "openai".to_string(),
            })
        })
    }
}

fn openai_endpoint(raw: &str) -> String {
    let trimmed = raw.trim().trim_end_matches('/');
    if trimmed.ends_with("/v1") {
        trimmed.to_string()
    } else {
        format!("{trimmed}/v1")
    }
}
