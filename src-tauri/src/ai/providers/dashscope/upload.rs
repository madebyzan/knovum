use futures::future::BoxFuture;
use reqwest::multipart::{Form, Part};
use serde::Deserialize;

use super::endpoint;
use crate::ai::{
    contracts::{FileUploadCapability, UploadedFileReference},
    transport::{http_client, response, SecurityContext},
};
use crate::foundation::app_error::{AppError, AppResult};

pub struct DashScopeFileUploadAdapter;

#[derive(Deserialize)]
struct PolicyEnvelope {
    data: Option<UploadPolicy>,
    code: Option<String>,
    message: Option<String>,
}

#[derive(Deserialize)]
struct UploadPolicy {
    policy: String,
    signature: String,
    upload_dir: String,
    upload_host: String,
    oss_access_key_id: String,
    x_oss_object_acl: String,
    x_oss_forbid_overwrite: String,
}

/// DashScope multimodal requests reference temporary files by an `oss://`
/// path obtained through the documented get-policy + OSS form-upload flow,
/// resolved server-side via the `X-DashScope-OssResourceResolve` header.
impl FileUploadCapability for DashScopeFileUploadAdapter {
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
            let policy_url = format!(
                "{}/uploads?action=getPolicy&model={}",
                endpoint(&ctx.endpoint),
                ctx.model
            );
            let policy_response = client
                .get(&policy_url)
                .bearer_auth(api_key)
                .send()
                .await
                .map_err(provider_err)?;
            if !policy_response.status().is_success() {
                return Err(AppError::new(
                    "provider_request_failed",
                    response::read_text(policy_response).await?,
                ));
            }
            let envelope: PolicyEnvelope =
                response::read_json(policy_response, response::MAX_CONTROL_JSON_BYTES).await?;
            let policy = envelope.data.ok_or_else(|| {
                AppError::new(
                    "provider_request_failed",
                    format!(
                        "DashScope upload policy failed: {} - {}",
                        envelope.code.unwrap_or_else(|| "unknown".to_string()),
                        envelope.message.unwrap_or_else(|| "no policy".to_string()),
                    ),
                )
            })?;

            let object_key = format!("{}/{}", policy.upload_dir, file_name);
            let part = Part::bytes(bytes)
                .file_name(file_name.to_string())
                .mime_str(mime_type)
                .map_err(|error| AppError::new("invalid_request", error.to_string()))?;
            let form = Form::new()
                .text("OSSAccessKeyId", policy.oss_access_key_id)
                .text("Signature", policy.signature)
                .text("policy", policy.policy)
                .text("key", object_key.clone())
                .text("x-oss-object-acl", policy.x_oss_object_acl)
                .text("x-oss-forbid-overwrite", policy.x_oss_forbid_overwrite)
                .text("success_action_status", "200")
                .part("file", part);

            let upload_response = client
                .post(&policy.upload_host)
                .multipart(form)
                .send()
                .await
                .map_err(provider_err)?;
            if !upload_response.status().is_success() {
                return Err(AppError::new(
                    "provider_request_failed",
                    response::read_text(upload_response).await?,
                ));
            }

            Ok(UploadedFileReference {
                file_id: object_key.clone(),
                access_url: Some(format!("oss://{object_key}")),
                provider: "dashscope".to_string(),
            })
        })
    }
}

fn provider_err(error: reqwest::Error) -> AppError {
    AppError::new("provider_request_failed", error.to_string())
}
