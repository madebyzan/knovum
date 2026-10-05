use reqwest::multipart::{Form, Part};

use super::{http_client, response, SecurityContext};
use crate::app_error::{AppError, AppResult};

pub(crate) struct UploadFile<'a> {
    pub bytes: Vec<u8>,
    pub mime_type: &'a str,
    pub file_name: &'a str,
}

pub(crate) async fn upload_file(
    context: &SecurityContext,
    url: &str,
    file: UploadFile<'_>,
    purpose: &str,
) -> AppResult<serde_json::Value> {
    let api_key = context.require_api_key()?;
    let part = Part::bytes(file.bytes)
        .file_name(file.file_name.to_owned())
        .mime_str(file.mime_type)
        .map_err(|error| AppError::new("invalid_request", error.to_string()))?;
    let form = Form::new()
        .part("file", part)
        .text("purpose", purpose.to_owned());
    let result = http_client()
        .post(url)
        .bearer_auth(api_key)
        .multipart(form)
        .send()
        .await
        .map_err(|error| AppError::new("provider_request_failed", error.to_string()))?;
    if !result.status().is_success() {
        return Err(AppError::new(
            "provider_request_failed",
            response::read_text(result).await?,
        ));
    }
    response::read_json(result, response::MAX_CONTROL_JSON_BYTES).await
}
