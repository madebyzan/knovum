
use super::*;

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
fn remote_resources_reject_private_and_special_addresses() {
    for address in [
        "127.0.0.1",
        "10.0.0.1",
        "172.16.0.1",
        "192.168.0.1",
        "169.254.169.254",
        "100.64.0.1",
        "::1",
        "fc00::1",
        "fe80::1",
        "::ffff:127.0.0.1",
        "fec0::1",
    ] {
        assert!(!public_address(address.parse().unwrap()), "{address}");
    }
    assert!(public_address("8.8.8.8".parse().unwrap()));
    assert!(public_address("2606:4700:4700::1111".parse().unwrap()));
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
