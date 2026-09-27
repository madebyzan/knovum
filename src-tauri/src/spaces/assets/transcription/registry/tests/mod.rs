use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn inference_wait_can_be_cancelled_and_releases_after_execution() {
    let first_token = CancellationToken::new();
    let first = super::execution_permit(&first_token).await.unwrap();
    let waiting = CancellationToken::new();
    waiting.cancel();
    let released_permit = super::execution_permit(&waiting).await;
    assert_eq!(
        released_permit.unwrap_err(),
        crate::media::TRANSCRIPTION_CANCELLED
    );
    drop(first);
    let next = super::execution_permit(&first_token).await;
    assert!(next.is_ok());
}
