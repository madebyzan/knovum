use crate::ai::transport::response;
use futures::future::BoxFuture;

use crate::ai::{
    contracts::{FileUploadCapability, UploadedFileReference},
    transport::{http_client, SecurityContext},
};
use crate::app_error::{AppError, AppResult};

pub struct GeminiFileUploadAdapter;

impl FileUploadCapability for GeminiFileUploadAdapter {
    fn upload_ephemeral_file<'a>(
        &'a self,
        ctx: &'a SecurityContext,
        bytes: Vec<u8>,
        mime_type: &'a str,
        file_name: &'a str,
    ) -> BoxFuture<'a, AppResult<UploadedFileReference>> {
        Box::pin(async move {
            let api_key = ctx.require_api_key()?;
            let client = http_client();
            let endpoint = ctx.endpoint.trim_end_matches('/');
            let host = endpoint
                .trim_end_matches("/v1beta")
                .trim_end_matches("/v1")
                .trim_end_matches('/');
            let num_bytes = bytes.len();
            let start_url = format!("{host}/upload/v1beta/files");

            let metadata_payload = serde_json::json!({
                "file": {
                    "display_name": file_name
                }
            });

            let init_response = client
                .post(&start_url)
                .header("x-goog-api-key", api_key)
                .header("X-Goog-Upload-Protocol", "resumable")
                .header("X-Goog-Upload-Command", "start")
                .header("X-Goog-Upload-Header-Content-Length", num_bytes.to_string())
                .header("X-Goog-Upload-Header-Content-Type", mime_type)
                .header("Content-Type", "application/json")
                .json(&metadata_payload)
                .send()
                .await
                .map_err(|err| AppError::new("provider_request_failed", err.to_string()))?;

            if !init_response.status().is_success() {
                let error_text = response::read_text(init_response).await?;
                return Err(AppError::new("provider_request_failed", error_text));
            }

            let upload_url = init_response
                .headers()
                .get("x-goog-upload-url")
                .and_then(|v| v.to_str().ok())
                .ok_or_else(|| {
                    AppError::new(
                        "invalid_response",
                        "Gemini upload did not return x-goog-upload-url",
                    )
                })?
                .to_string();

            let upload_response = client
                .post(&upload_url)
                .header("X-Goog-Upload-Offset", "0")
                .header("X-Goog-Upload-Command", "upload, finalize")
                .header("Content-Type", mime_type)
                .body(bytes)
                .send()
                .await
                .map_err(|err| AppError::new("provider_request_failed", err.to_string()))?;

            if !upload_response.status().is_success() {
                let error_text = response::read_text(upload_response).await?;
                return Err(AppError::new("provider_request_failed", error_text));
            }

            let file_data: serde_json::Value =
                response::read_json(upload_response, response::MAX_CONTROL_JSON_BYTES).await?;

            let file_id = file_data["file"]["name"]
                .as_str()
                .ok_or_else(|| {
                    AppError::new("invalid_response", "Missing file name in Gemini response")
                })?
                .to_string();

            let access_url = file_data["file"]["uri"].as_str().map(ToString::to_string);

            Ok(UploadedFileReference {
                file_id,
                access_url,
                provider: "gemini".to_string(),
            })
        })
    }
}
