use base64::{engine::general_purpose::STANDARD, Engine};

use crate::ai::{
    contracts::{FileUploadCapability, ModelBinarySource, ModelContentPart},
    error::{AiError, AiResult},
    models::ExecutionTarget,
    providers::resolve_provider,
    transport::SecurityContext,
};

/// The caller owns local-file authorization and content validation.
pub(crate) async fn prepare_image_input(
    target: &ExecutionTarget,
    bytes: Vec<u8>,
    media_type: &str,
    name: String,
) -> AiResult<ModelContentPart> {
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
) -> AiResult<ModelBinarySource> {
    let uploaded = uploader
        .upload_ephemeral_file(context, bytes, media_type, name)
        .await?;
    let url = uploaded
        .access_url
        .ok_or_else(|| AiError::invalid("Model media upload returned no consumable URI"))?;
    let parsed = reqwest::Url::parse(&url).map_err(|error| AiError::invalid(error.to_string()))?;
    if parsed.scheme() != "https"
        || parsed.host_str().is_none()
        || !parsed.username().is_empty()
        || parsed.password().is_some()
    {
        return Err(AiError::invalid(
            "Model media upload must return a credential-free HTTPS URI",
        ));
    }
    Ok(ModelBinarySource::Url { url })
}

#[cfg(test)]
mod tests {
    use crate::ai::{contracts::ProviderKind, providers::resolve_provider};

    #[test]
    fn upload_support_does_not_imply_model_turn_can_consume_the_upload() {
        for provider in [
            ProviderKind::OpenAi,
            ProviderKind::DashScope,
            ProviderKind::Grok,
            ProviderKind::Moonshot,
            ProviderKind::Zhipu,
        ] {
            let provider = resolve_provider(provider);
            assert!(provider.as_file_upload().is_some());
            assert!(provider.as_input_upload().is_none());
        }
        assert!(resolve_provider(ProviderKind::Gemini)
            .as_input_upload()
            .is_some());
    }

    struct Uploader(Option<&'static str>);
    impl crate::ai::contracts::FileUploadCapability for Uploader {
        fn upload_ephemeral_file<'a>(
            &'a self,
            _: &'a crate::ai::transport::SecurityContext,
            _: Vec<u8>,
            _: &'a str,
            _: &'a str,
        ) -> futures::future::BoxFuture<
            'a,
            crate::ai::error::AiResult<crate::ai::contracts::UploadedFileReference>,
        > {
            Box::pin(async move {
                let uri = self
                    .0
                    .ok_or_else(|| crate::ai::error::AiError::invalid("upload failed"))?;
                Ok(crate::ai::contracts::UploadedFileReference {
                    file_id: "files/source".into(),
                    access_url: Some(uri.into()),
                    provider: "test".into(),
                })
            })
        }
    }

    #[tokio::test]
    async fn uploaded_sources_preserve_the_uri_and_failures_do_not_become_inline_images() {
        let context = crate::ai::transport::SecurityContext {
            provider: "test".into(),
            model: "test".into(),
            endpoint: "https://unused.invalid".into(),
            api_key: None,
            temperature: 0.0,
        };
        let source = super::upload_image_reference(
            &Uploader(Some("https://media.invalid/files/source")),
            &context,
            vec![1],
            "image/png",
            "source.png",
        )
        .await
        .unwrap();
        assert!(matches!(
            source,
            crate::ai::contracts::ModelBinarySource::Url { .. }
        ));
        for uri in [
            None,
            Some("file:///private/source.png"),
            Some("https://secret@media.invalid/source"),
        ] {
            assert!(super::upload_image_reference(
                &Uploader(uri),
                &context,
                vec![1],
                "image/png",
                "source.png"
            )
            .await
            .is_err());
        }
    }
}
