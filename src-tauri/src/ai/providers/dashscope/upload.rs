use crate::ai::transport::response;
use futures::future::BoxFuture;
use reqwest::multipart::{Form, Part};

use super::image::dashscope_endpoint;
use crate::ai::{
    contracts::{FileUploadCapability, UploadedFileReference},
    transport::{http_client, SecurityContext},
};
use crate::app_error::{AppError, AppResult};

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
            let secret = ctx.require_api_key()?;
            let client = http_client();
            let endpoint = dashscope_endpoint(&ctx.endpoint)?;
            let upload_url = format!("{endpoint}/files");

            let part = Part::bytes(bytes)
                .file_name(file_name.to_string())
                .mime_str(mime_type)
                .map_err(|e| AppError::new("invalid_request", e.to_string()))?;

            let form = Form::new().part("file", part).text("purpose", "multimodal");

            let upload_response = client
                .post(&upload_url)
                .bearer_auth(secret)
                .multipart(form)
                .send()
                .await
                .map_err(|e| AppError::new("provider_request_failed", e.to_string()))?;

            if !upload_response.status().is_success() {
                let error_text = response::read_text(upload_response).await?;
                return Err(AppError::new("provider_request_failed", error_text));
            }

            let file_data: serde_json::Value =
                response::read_json(upload_response, response::MAX_CONTROL_JSON_BYTES).await?;

            let file_id = file_data["file_id"]
                .as_str()
                .or_else(|| file_data["id"].as_str())
                .ok_or_else(|| {
                    AppError::new("invalid_response", "Missing file ID in DashScope response")
                })?
                .to_string();

            let access_url = file_data["url"].as_str().map(ToString::to_string);

            Ok(UploadedFileReference {
                file_id,
                access_url,
                provider: "dashscope".to_string(),
            })
        })
    }
}
