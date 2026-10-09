use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

use super::{decode_base64, read_bytes, read_json};

#[test]
fn inline_media_enforces_encoded_and_decoded_size_limits() {
    assert_eq!(decode_base64("MTIzNDU=", 5).unwrap(), b"12345");
    assert!(decode_base64("MTIzNDU=", 4).is_err());
    assert!(decode_base64("MTIzNDU2Nw==", 4).is_err());
    assert!(decode_base64("invalid", 4).is_err());
}

async fn response(wire: &'static str) -> reqwest::Response {
    let server = TcpListener::bind("127.0.0.1:0").expect("local server");
    let address = server.local_addr().expect("address");
    std::thread::spawn(move || {
        let (mut socket, _) = server.accept().expect("connection");
        let mut request = BufReader::new(&mut socket);
        loop {
            let mut header = String::new();
            assert!(request.read_line(&mut header).expect("request") > 0);
            if header == "\r\n" {
                break;
            }
        }
        drop(request);
        socket.write_all(wire.as_bytes()).expect("response");
    });
    reqwest::Client::new()
        .get(format!("http://{address}/"))
        .send()
        .await
        .expect("response")
}

#[tokio::test]
async fn rejects_declared_and_chunked_oversized_bodies() {
    for wire in [
        "HTTP/1.1 200 OK\r\nContent-Length: 8\r\nConnection: close\r\n\r\n12345678",
        "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n4\r\n1234\r\n4\r\n5678\r\n0\r\n\r\n",
    ] {
        let error = read_bytes(response(wire).await, 7).await.expect_err("size limit");
        assert_eq!(error.code, "invalid_response");
    }
}

#[tokio::test]
async fn accepts_the_exact_limit_and_rejects_malformed_json() {
    let wire = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}";
    let value: serde_json::Value = read_json(response(wire).await, 2)
        .await
        .expect("exact limit");
    assert_eq!(value, serde_json::json!({}));
    let malformed = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{{";
    read_json::<serde_json::Value>(response(malformed).await, 2)
        .await
        .expect_err("malformed JSON");
}
