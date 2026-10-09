use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

use super::http_client;

#[tokio::test]
async fn custom_api_keys_cannot_follow_a_cross_origin_redirect() {
    let destination = TcpListener::bind("127.0.0.1:0").unwrap();
    destination.set_nonblocking(true).unwrap();
    let origin = TcpListener::bind("127.0.0.1:0").unwrap();
    let origin_address = origin.local_addr().unwrap();
    let destination_address = destination.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let (mut connection, _) = origin.accept().unwrap();
        let mut input = BufReader::new(&mut connection);
        let mut headers = String::new();
        loop {
            let mut line = String::new();
            assert!(input.read_line(&mut line).unwrap() > 0);
            headers.push_str(&line);
            if line == "\r\n" {
                break;
            }
        }
        assert!(headers
            .to_ascii_lowercase()
            .contains("x-goog-api-key: test-secret"));
        drop(input);
        write!(connection, "HTTP/1.1 302 Found\r\nLocation: http://{destination_address}/collect\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
    });
    let request = http_client()
        .get(format!("http://{origin_address}/"))
        .header("x-goog-api-key", "test-secret")
        .send()
        .await;
    assert!(request.unwrap_err().is_redirect());
    server.join().unwrap();
    assert_eq!(
        destination.accept().unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock
    );
}
