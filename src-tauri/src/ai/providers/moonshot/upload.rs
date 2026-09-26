use crate::ai::transport::response;
use futures::future::BoxFuture;
use reqwest::multipart::{Form, Part};

use crate::ai::{
    contracts::{FileUploadCapability, UploadedFileReference},
    error::{AiError, AiErrorCode, AiResult},
    transport::{http_client, SecurityContext},
};

pub struct MoonshotFileUploadAdapter;

impl FileUploadCapability for MoonshotFileUploadAdapter {
    fn upload_ephemeral_file<'a>(
        &'a self,
        ctx: &'a SecurityContext,
        bytes: Vec<u8>,
        mime_type: &'a str,
        file_name: &'a str,
    ) -> BoxFuture<'a, AiResult<UploadedFileReference>> {
        Box::pin(async move {
            let api_key = ctx.require_api_key()?;
            let client = http_client();
            let endpoint = moonshot_endpoint(&ctx.endpoint);
            let upload_url = format!("{endpoint}/files");

            let part = Part::bytes(bytes)
                .file_name(file_name.to_string())
                .mime_str(mime_type)
                .map_err(|e| AiError::new(AiErrorCode::InvalidRequest, e.to_string()))?;

            let form = Form::new()
                .part("file", part)
                .text("purpose", "file-extract");

            let upload_response = client
                .post(&upload_url)
                .bearer_auth(api_key)
                .multipart(form)
                .send()
                .await
                .map_err(|e| AiError::new(AiErrorCode::ProviderRequestFailed, e.to_string()))?;

            if !upload_response.status().is_success() {
                let error_text = response::read_text(upload_response).await?;
                return Err(AiError::new(AiErrorCode::ProviderRequestFailed, error_text));
            }

            let file_data: serde_json::Value =
                response::read_json(upload_response, response::MAX_CONTROL_JSON_BYTES).await?;

            let file_id = file_data["id"]
                .as_str()
                .ok_or_else(|| {
                    AiError::new(
                        AiErrorCode::InvalidResponse,
                        "Missing file ID in Moonshot response",
                    )
                })?
                .to_string();

            Ok(UploadedFileReference {
                file_id,
                access_url: None,
                provider: "moonshot".to_string(),
            })
        })
    }
}

fn moonshot_endpoint(raw: &str) -> String {
    let trimmed = raw.trim().trim_end_matches('/');
    if trimmed.ends_with("/v1") {
        trimmed.to_string()
    } else {
        format!("{trimmed}/v1")
    }
}
