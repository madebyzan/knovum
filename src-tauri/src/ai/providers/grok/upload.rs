use futures::future::BoxFuture;
use reqwest::multipart::{Form, Part};
use serde::Deserialize;

use crate::ai::{
    contracts::{FileUploadCapability, UploadedFileReference},
    transport::{http_client, response, SecurityContext},
};
use crate::foundation::app_error::{AppError, AppResult};

pub struct GrokFileUploadAdapter;

#[derive(Deserialize)]
struct UploadedFile {
    id: String,
}

impl FileUploadCapability for GrokFileUploadAdapter {
    fn upload_ephemeral_file<'a>(
        &'a self,
        ctx: &'a SecurityContext,
        bytes: Vec<u8>,
        mime_type: &'a str,
        file_name: &'a str,
    ) -> BoxFuture<'a, AppResult<UploadedFileReference>> {
        Box::pin(async move {
            let image = Part::bytes(bytes)
                .file_name(file_name.to_owned())
                .mime_str(mime_type)
                .map_err(|error| AppError::new("invalid_request", error.to_string()))?;
            let form = Form::new()
                .text("expires_after", "3600")
                .part("file", image);
            let request = http_client()
                .post(format!("{}/files", ctx.endpoint.trim_end_matches('/')))
                .bearer_auth(ctx.require_api_key()?)
                .multipart(form)
                .send()
                .await
                .map_err(|error| AppError::new("provider_request_failed", error.to_string()))?;
            if !request.status().is_success() {
                return Err(AppError::new(
                    "provider_request_failed",
                    response::read_text(request).await?,
                ));
            }
            let uploaded: UploadedFile =
                response::read_json(request, response::MAX_CONTROL_JSON_BYTES).await?;
            if uploaded.id.is_empty() {
                return Err(AppError::new(
                    "invalid_response",
                    "xAI image upload returned no file ID",
                ));
            }
            Ok(UploadedFileReference {
                file_id: uploaded.id,
                access_url: None,
                provider: "grok".into(),
            })
        })
    }
}
