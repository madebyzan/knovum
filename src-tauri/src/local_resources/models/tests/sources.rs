use super::super::{catalog, download, sources};
use std::io::{Read, Write};
use tokio_util::sync::CancellationToken;

#[test]
fn endpoints_preserve_the_same_pinned_artifact() {
    let descriptor = catalog::TRANSCRIPTION;
    let urls = sources::urls(
        descriptor,
        &["https://huggingface.co", "https://hf-mirror.com/"],
    )
    .unwrap();
    assert_eq!(urls[0].path(), urls[1].path());
    assert!(urls[0].path().contains(descriptor.revision));
    assert!(urls[0].path().ends_with(descriptor.file));
    assert_eq!(urls[1].host_str(), Some("hf-mirror.com"));
    assert_eq!(urls[1].query(), Some("download=true"));
    let custom = sources::urls(descriptor, &["https://models.example/hub/"]).unwrap();
    assert_eq!(custom.len(), 1);
    assert!(custom[0].path().starts_with("/hub/ggerganov/"));
}

#[test]
fn unsafe_or_malformed_endpoints_are_rejected() {
    for endpoint in [
        "",
        "http://example.com",
        "https://user:secret@example.com",
        "https://example.com?token=x",
        "https://example.com/#fragment",
    ] {
        assert!(sources::urls(catalog::TRANSCRIPTION, &[endpoint]).is_err());
    }
}

fn server(status: &str) -> (reqwest::Url, std::thread::JoinHandle<()>) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let url =
        reqwest::Url::parse(&format!("http://{}/model", listener.local_addr().unwrap())).unwrap();
    let response =
        format!("HTTP/1.1 {status}\r\nContent-Length: 5\r\nConnection: close\r\n\r\nmodel");
    let thread = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .unwrap();
        let _ = stream.read(&mut [0; 4096]).unwrap();
        stream.write_all(response.as_bytes()).unwrap();
    });
    (url, thread)
}

#[tokio::test]
async fn unavailable_primary_uses_next_source() {
    let (primary, first) = server("503 Service Unavailable");
    let (mirror, second) = server("200 OK");
    let response = download::open_response(
        &reqwest::Client::builder().no_proxy().build().unwrap(),
        vec![primary, mirror.clone()],
        &CancellationToken::new(),
    )
    .await
    .unwrap();
    assert_eq!(response.url(), &mirror);
    assert_eq!(response.bytes().await.unwrap().as_ref(), b"model");
    first.join().unwrap();
    second.join().unwrap();
}

#[tokio::test]
async fn cancellation_does_not_try_another_source() {
    let cancellation = CancellationToken::new();
    cancellation.cancel();
    let error = download::open_response(
        &reqwest::Client::new(),
        vec![reqwest::Url::parse("http://127.0.0.1:1").unwrap()],
        &cancellation,
    )
    .await
    .unwrap_err();
    assert_eq!(error, "Model download cancelled");
}
