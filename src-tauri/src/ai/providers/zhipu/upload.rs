use futures::future::BoxFuture;
use reqwest::multipart::{Form, Part};

use crate::ai::{
    contracts::{FileUploadCapability, UploadedFileReference},
    error::{AiError, AiErrorCode, AiResult},
    transport::{http_client, SecurityContext},
};

pub struct ZhipuFileUploadAdapter;

impl FileUploadCapability for ZhipuFileUploadAdapter {
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
            let endpoint = ctx.endpoint.trim_end_matches('/');
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
                let error_text = upload_response.text().await.unwrap_or_default();
                return Err(AiError::new(AiErrorCode::ProviderRequestFailed, error_text));
            }

            let file_data: serde_json::Value = upload_response
                .json()
                .await
                .map_err(|e| AiError::new(AiErrorCode::InvalidResponse, e.to_string()))?;

            let file_id = file_data["id"]
                .as_str()
                .ok_or_else(|| {
                    AiError::new(
                        AiErrorCode::InvalidResponse,
                        "Missing file ID in Zhipu response",
                    )
                })?
                .to_string();

            let access_url = file_data["url"].as_str().map(ToString::to_string);

            Ok(UploadedFileReference {
                file_id,
                access_url,
                provider: "zhipu".to_string(),
            })
        })
    }
}
