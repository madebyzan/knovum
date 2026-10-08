use super::read_file_bounded;

#[tokio::test]
async fn file_reader_rejects_bytes_beyond_its_limit() {
    let path = std::env::temp_dir().join(format!("knovum-upload-test-{}", uuid::Uuid::new_v4()));
    tokio::fs::write(&path, b"12345").await.unwrap();
    let read_attempt = read_file_bounded(&path, 4).await;
    tokio::fs::remove_file(&path).await.unwrap();
    assert!(read_attempt.is_err());
}
