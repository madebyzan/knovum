use super::*;
use crate::ai::contracts::{
    GeneratedVideo, VideoGenerationCapability, VideoGenerationInput, VideoGenerationStageCallback,
    VideoGenerationStatus, VideoGenerationSubmission,
};
use futures::future::BoxFuture;

struct PublicResultAdapter;

impl VideoGenerationCapability for PublicResultAdapter {
    fn submit<'a>(
        &'a self,
        _ctx: &'a SecurityContext,
        _req: &'a VideoGenerationInput,
    ) -> BoxFuture<'a, AppResult<VideoGenerationSubmission>> {
        unimplemented!("download tests never submit")
    }

    fn status<'a>(
        &'a self,
        _ctx: &'a SecurityContext,
        _operation_id: &'a str,
    ) -> BoxFuture<'a, AppResult<VideoGenerationStatus>> {
        unimplemented!("download tests never poll")
    }

    fn generate<'a>(
        &'a self,
        _ctx: &'a SecurityContext,
        _req: &'a VideoGenerationInput,
        _cancellation: &'a CancellationToken,
        _stage: VideoGenerationStageCallback<'a>,
    ) -> BoxFuture<'a, AppResult<GeneratedVideo>> {
        unimplemented!("download tests never generate")
    }
}

#[tokio::test]
async fn cancelled_download_never_resolves_or_decodes_its_source() {
    let cancellation = CancellationToken::new();
    cancellation.cancel();
    let ctx = SecurityContext {
        provider: "gemini".into(),
        model: "veo".into(),
        endpoint: "https://example.com/v1".into(),
        api_key: None,
        temperature: 1.0,
    };
    assert_eq!(
        materialize(&PublicResultAdapter, &ctx, "invalid-url", &cancellation)
            .await
            .unwrap_err()
            .code,
        "cancelled"
    );
}
