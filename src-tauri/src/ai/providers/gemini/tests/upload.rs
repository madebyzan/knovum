use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener};

use super::upload::GeminiFileUploadAdapter;
use crate::ai::{contracts::FileUploadCapability, transport::SecurityContext};

#[tokio::test]
async fn upload_does_not_send_file_bytes_to_a_returned_cross_origin_url() {
    let destination = TcpListener::bind("127.0.0.1:0").unwrap();
    destination.set_nonblocking(true).unwrap();
    let destination_address = destination.local_addr().unwrap();
    let (context, server) = upload_origin(move |_| format!("http://{destination_address}/upload"));
    let error = GeminiFileUploadAdapter
        .upload_ephemeral_file(
            &context,
            b"private file bytes".to_vec(),
            "text/plain",
            "file.txt",
        )
        .await
        .unwrap_err();
    assert_eq!(error.code, "invalid_response");
    server.join().unwrap();
    assert_eq!(
        destination.accept().unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock
    );
}

#[tokio::test]
async fn upload_transport_error_does_not_expose_the_upload_session() {
    let (context, server) = upload_origin(|address| {
        format!("http://{address}/upload?upload_id=private-session-marker")
    });
    let error = GeminiFileUploadAdapter
        .upload_ephemeral_file(
            &context,
            b"private file bytes".to_vec(),
            "text/plain",
            "file.txt",
        )
        .await
        .unwrap_err();
    server.join().unwrap();
    assert_eq!(error.code, "provider_request_failed");
    assert!(!error.message.contains("private-session-marker"));
}

fn upload_origin(
    upload_url: impl FnOnce(SocketAddr) -> String,
) -> (SecurityContext, std::thread::JoinHandle<()>) {
    let origin = TcpListener::bind("127.0.0.1:0").unwrap();
    let origin_address = origin.local_addr().unwrap();
    let upload_url = upload_url(origin_address);
    let server = std::thread::spawn(move || {
        let (mut connection, _) = origin.accept().unwrap();
        let mut input = BufReader::new(&mut connection);
        let mut length = 0;
        loop {
            let mut line = String::new();
            assert!(input.read_line(&mut line).unwrap() > 0);
            if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                length = value.trim().parse::<usize>().unwrap();
            }
            if line == "\r\n" {
                break;
            }
        }
        let mut metadata = vec![0; length];
        input.read_exact(&mut metadata).unwrap();
        drop(input);
        write!(connection, "HTTP/1.1 200 OK\r\nx-goog-upload-url: {upload_url}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
    });
    let context = SecurityContext {
        provider: "gemini".into(),
        model: "gemini-test".into(),
        endpoint: format!("http://{origin_address}/v1beta"),
        api_key: Some("test-key".into()),
        temperature: 0.0,
    };
    (context, server)
}
