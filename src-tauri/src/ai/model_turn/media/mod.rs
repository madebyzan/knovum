use base64::{engine::general_purpose::STANDARD, Engine};

use crate::ai::{
    contracts::{FileUploadCapability, ModelBinarySource, ModelContentPart},
    models::ModelExecutionTarget,
    providers::resolve_provider,
    transport::SecurityContext,
};
use crate::app_error::{AppError, AppResult};

/// The caller owns local-file authorization and content validation.
pub(crate) async fn prepare_image_input(
    target: &ModelExecutionTarget,
    bytes: Vec<u8>,
    media_type: &str,
    name: String,
) -> AppResult<ModelContentPart> {
    let provider = resolve_provider(target.model_provider_kind()?);
    let source = match provider.as_input_upload() {
        Some(uploader) => {
            let context = SecurityContext::from_target(target).await?;
            upload_image_reference(uploader, &context, bytes, media_type, &name).await?
        }
        None => ModelBinarySource::Base64 {
            data: STANDARD.encode(bytes),
        },
    };
    Ok(ModelContentPart::Binary {
        media_type: media_type.to_owned(),
        source,
        name: Some(name),
    })
}

async fn upload_image_reference(
    uploader: &dyn FileUploadCapability,
    context: &SecurityContext,
    bytes: Vec<u8>,
    media_type: &str,
    name: &str,
) -> AppResult<ModelBinarySource> {
    let uploaded = uploader
        .upload_ephemeral_file(context, bytes, media_type, name)
        .await?;
    let url = uploaded
        .access_url
        .ok_or_else(|| AppError::invalid("Model media upload returned no consumable URI"))?;
    let parsed = reqwest::Url::parse(&url).map_err(|error| AppError::invalid(error.to_string()))?;
    if parsed.scheme() != "https"
        || parsed.host_str().is_none()
        || !parsed.username().is_empty()
        || parsed.password().is_some()
    {
        return Err(AppError::invalid(
            "Model media upload must return a credential-free HTTPS URI",
        ));
    }
    Ok(ModelBinarySource::Url { url })
}

#[cfg(test)]
mod tests;
