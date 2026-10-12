use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::test]
async fn clients_built_from_the_shared_builder_identify_knovum() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut request = Vec::new();
        let mut buffer = [0; 1024];
        loop {
            let read = socket.read(&mut buffer).await.unwrap();
            assert!(read > 0 && request.len() < 8192);
            request.extend_from_slice(&buffer[..read]);
            if request.ends_with(b"\r\n\r\n") {
                break;
            }
        }
        socket
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
            .await
            .unwrap();
        String::from_utf8(request).unwrap().to_ascii_lowercase()
    });
    super::client_builder()
        .build()
        .unwrap()
        .get(format!("http://{address}/probe"))
        .send()
        .await
        .unwrap();
    let expected = format!("user-agent: {}", super::USER_AGENT.to_ascii_lowercase());
    assert!(server.await.unwrap().contains(&expected));
}
