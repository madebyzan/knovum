use super::*;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

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
async fn workspace_links_do_not_copy_and_external_files_are_rejected() {
    let root = std::env::temp_dir().join(format!("agent-links-{}", uuid::Uuid::new_v4()));
    tokio::fs::create_dir(&root).await.unwrap();
    let file = root.join("report.md");
    tokio::fs::write(&file, "# Report").await.unwrap();
    let uri = tauri::Url::from_file_path(&file).unwrap();
    acquire(&root, ResourceLink::new("Report", uri.to_string()), None)
        .await
        .unwrap();
    assert_eq!(
        tokio::fs::read_dir(&root)
            .await
            .unwrap()
            .next_entry()
            .await
            .unwrap()
            .unwrap()
            .path(),
        file
    );
    let outside = root.with_extension("txt");
    tokio::fs::write(&outside, "private").await.unwrap();
    let uri = tauri::Url::from_file_path(&outside).unwrap();
    assert!(
        acquire(&root, ResourceLink::new("Outside", uri.to_string()), None)
            .await
            .is_err()
    );
    assert!(require_https(&tauri::Url::parse("http://example.com/a.png").unwrap()).is_err());
    tokio::fs::remove_dir_all(root).await.unwrap();
    tokio::fs::remove_file(outside).await.unwrap();
}
#[test]
fn remote_type_uses_final_url_and_response_content_type() {
    assert_eq!(
        remote_extension(
            &tauri::Url::parse("https://cdn.example.com/final.png").unwrap(),
            Some("image/png"),
            Some("application/octet-stream"),
        )
        .unwrap(),
        "png"
    );
    let jpeg_extension = remote_extension(
        &tauri::Url::parse("https://cdn.example.com/download.php").unwrap(),
        Some("image/jpeg; charset=binary"),
        Some("application/octet-stream"),
    )
    .unwrap();
    assert!(mime_guess::get_mime_extensions_str("image/jpeg")
        .unwrap()
        .contains(&jpeg_extension.as_str()));
}

#[tokio::test]
async fn oversized_remote_resources_leave_no_partial_file() {
    for wire in [
        "HTTP/1.1 200 OK\r\nContent-Length: 8\r\nConnection: close\r\n\r\n12345678",
        "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n4\r\n1234\r\n4\r\n5678\r\n0\r\n\r\n",
    ] {
        let workspace =
            std::env::temp_dir().join(format!("agent-link-limit-{}", uuid::Uuid::new_v4()));
        tokio::fs::create_dir(&workspace).await.unwrap();
        persist_response(&workspace, response(wire).await, "output", "bin", 7)
            .await
            .expect_err("size limit");
        assert!(tokio::fs::read_dir(&workspace)
            .await
            .unwrap()
            .next_entry()
            .await
            .unwrap()
            .is_none());
        tokio::fs::remove_dir_all(workspace).await.unwrap();
    }
}

#[test]
fn remote_limits_follow_the_resource_type() {
    assert_eq!(
        remote_size_limit(Some("image/png"), None),
        MAX_REMOTE_IMAGE_BYTES
    );
    assert_eq!(
        remote_size_limit(Some("audio/wav"), None),
        MAX_REMOTE_AUDIO_BYTES
    );
    assert_eq!(
        remote_size_limit(Some("video/mp4"), None),
        MAX_REMOTE_VIDEO_BYTES
    );
    assert_eq!(
        remote_size_limit(Some("application/octet-stream"), Some("video/mp4")),
        MAX_REMOTE_VIDEO_BYTES
    );
    assert_eq!(
        remote_size_limit(Some("application/json"), Some("video/mp4")),
        MAX_REMOTE_RESOURCE_BYTES
    );
}
