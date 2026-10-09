use super::*;
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
        materialize(&ctx, "invalid-url", &cancellation)
            .await
            .unwrap_err()
            .code,
        "cancelled"
    );
}
