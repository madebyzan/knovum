use super::super::{
    file_upload::{upload_file, UploadFile},
    SecurityContext,
};
use std::{
    io::{BufRead, BufReader, Read, Write},
    net::TcpListener,
};

async fn upload(
    status: &str,
    body: &str,
) -> crate::foundation::app_error::AppResult<serde_json::Value> {
    let server = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = server.local_addr().unwrap();
    let wire = format!(
        "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let request = std::thread::spawn(move || {
        let (mut socket, _) = server.accept().unwrap();
        let mut reader = BufReader::new(&mut socket);
        let mut headers = String::new();
        let mut length = 0;
        loop {
            let mut line = String::new();
            assert!(reader.read_line(&mut line).unwrap() > 0);
            headers.push_str(&line);
            if line == "\r\n" {
                break;
            }
            if let Some(value) = line.to_lowercase().strip_prefix("content-length:") {
                length = value.trim().parse::<usize>().unwrap();
            }
        }
        let mut bytes = vec![0; length];
        reader.read_exact(&mut bytes).unwrap();
        drop(reader);
        socket.write_all(wire.as_bytes()).unwrap();
        (headers, String::from_utf8(bytes).unwrap())
    });
    let context = SecurityContext {
        provider: "openai".into(),
        model: "test".into(),
        endpoint: format!("http://{address}"),
        api_key: Some("test-key".into()),
        temperature: 0.0,
    };
    let upload = upload_file(
        &context,
        &format!("http://{address}/files"),
        UploadFile {
            bytes: b"file contents".to_vec(),
            mime_type: "text/plain",
            file_name: "source.txt",
        },
        "assistants",
    )
    .await;
    let (headers, body) = request.join().unwrap();
    assert!(headers.starts_with("POST /files "));
    assert!(headers
        .to_lowercase()
        .contains("authorization: bearer test-key"));
    assert!(body.contains("filename=\"source.txt\""));
    assert!(body.contains("assistants"));
    assert!(body.contains("file contents"));
    upload
}

#[tokio::test]
async fn multipart_upload_retains_file_metadata_and_decodes_json() {
    assert_eq!(
        upload("200 OK", r#"{"id":"file-1"}"#).await.unwrap()["id"],
        "file-1"
    );
}

#[tokio::test]
async fn failures_keep_transport_error_codes_and_reject_malformed_json() {
    let error = upload("400 Bad Request", "rejected").await.unwrap_err();
    assert_eq!(error.code, "provider_request_failed");
    assert_eq!(error.message, "rejected");
    assert_eq!(
        upload("200 OK", "invalid json").await.unwrap_err().code,
        "invalid_response"
    );
}
