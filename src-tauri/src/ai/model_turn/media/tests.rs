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
