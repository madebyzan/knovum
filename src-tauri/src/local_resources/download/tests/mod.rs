use super::*;
use tokio::{io::AsyncReadExt, net::TcpListener};

async fn serve(listener: TcpListener, response: String) -> String {
    let (mut socket, _) = listener.accept().await.unwrap();
    let mut request = Vec::new();
    loop {
        let mut buffer = [0; 1024];
        let read = socket.read(&mut buffer).await.unwrap();
        assert!(read > 0 && request.len() < 8192);
        request.extend_from_slice(&buffer[..read]);
        if request.ends_with(b"\r\n\r\n") {
            break;
        }
    }
    socket.write_all(response.as_bytes()).await.unwrap();
    String::from_utf8(request).unwrap().to_ascii_lowercase()
}

fn destination() -> std::path::PathBuf {
    std::env::temp_dir().join(format!("knovum-transfer-{}", uuid::Uuid::new_v4()))
}

fn policy() -> DownloadPolicy {
    DownloadPolicy {
        maximum_bytes: 4,
        required_bytes: Some(4),
        cancellation_message: "cancelled",
    }
}

#[tokio::test]
async fn authorized_download_strips_credentials_on_cross_origin_redirect() {
    let origin = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let target = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/model", origin.local_addr().unwrap());
    let redirect = format!("HTTP/1.1 302 Found\r\nLocation: http://{}/file\r\nContent-Length: 0\r\nConnection: close\r\n\r\n", target.local_addr().unwrap());
    let origin_request = tokio::spawn(serve(origin, redirect));
    let target_request = tokio::spawn(serve(
        target,
        "HTTP/1.1 200 OK\r\nContent-Length: 4\r\nConnection: close\r\n\r\ndata".into(),
    ));
    let path = destination();
    let mut observed = Vec::new();
    to_path(
        reqwest::Client::new().get(url).bearer_auth("test-access"),
        &path,
        policy(),
        &CancellationToken::new(),
        &mut |bytes, _| observed.push(bytes),
    )
    .await
    .unwrap();
    assert!(origin_request
        .await
        .unwrap()
        .contains("authorization: bearer test-access"));
    assert!(!target_request.await.unwrap().contains("authorization:"));
    assert_eq!(std::fs::read(&path).unwrap(), b"data");
    assert_eq!(observed.last(), Some(&4));
    std::fs::remove_file(path).unwrap();
}

#[tokio::test]
async fn denied_access_preserves_http_status_without_creating_a_model_file() {
    let origin = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/model", origin.local_addr().unwrap());
    let server = tokio::spawn(serve(
        origin,
        "HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".into(),
    ));
    let path = destination();
    let result = to_path(
        reqwest::Client::new().get(url).bearer_auth("test-access"),
        &path,
        policy(),
        &CancellationToken::new(),
        &mut |_, _| {},
    )
    .await;
    assert!(matches!(
        result,
        Err(DownloadError::Http { status: 403, .. })
    ));
    assert!(!path.exists());
    server.await.unwrap();
}

#[tokio::test]
async fn cancellation_stops_before_sending_credentials_or_creating_a_file() {
    let cancellation = CancellationToken::new();
    cancellation.cancel();
    let path = destination();
    let result = to_path(
        reqwest::Client::new()
            .get("http://127.0.0.1:1/model")
            .bearer_auth("test-access"),
        &path,
        policy(),
        &cancellation,
        &mut |_, _| {},
    )
    .await;
    assert!(matches!(result, Err(DownloadError::Terminal(message)) if message == "cancelled"));
    assert!(!path.exists());
}

#[tokio::test]
async fn declared_oversize_is_rejected_before_creating_a_file() {
    let origin = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/model", origin.local_addr().unwrap());
    let server = tokio::spawn(serve(
        origin,
        "HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nlarge".into(),
    ));
    let path = destination();
    let result = to_path(
        reqwest::Client::new().get(url),
        &path,
        policy(),
        &CancellationToken::new(),
        &mut |_, _| {},
    )
    .await;
    assert!(matches!(result, Err(DownloadError::Terminal(_))));
    assert!(!path.exists());
    server.await.unwrap();
}
