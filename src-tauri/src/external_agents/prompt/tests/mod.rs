use super::*;

#[tokio::test]
async fn local_images_are_links_without_copying_or_encoding_the_source() {
    let directory = std::env::temp_dir().join(format!("agent-prompt-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&directory).unwrap();
    let source = directory.join("source image.png");
    std::fs::write(&source, b"local source").unwrap();
    let canonical = source.canonicalize().unwrap();
    let prompt =
        AgentPrompt::from_authorized_assets("Inspect this".into(), vec![canonical.clone()], |_| {
            Ok(())
        })
        .unwrap();
    let content = prompt.into_content().await.unwrap();
    assert!(content
        .iter()
        .all(|part| !matches!(part, ContentBlock::Image(_))));
    let link = content
        .iter()
        .find_map(|part| match part {
            ContentBlock::ResourceLink(link) => Some(link),
            _ => None,
        })
        .unwrap();
    // Compared as URLs: on Windows canonicalize() yields \\?\ verbatim paths that file URLs
    // never round-trip back to.
    assert_eq!(
        tauri::Url::parse(&link.uri).unwrap(),
        tauri::Url::from_file_path(&canonical).unwrap()
    );
    assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 1);
    std::fs::remove_dir_all(directory).unwrap();
}
