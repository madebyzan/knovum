use super::*;
use agent_client_protocol::schema::v1::{ResourceLink, TextResourceContents, ToolCallContent};

fn collected_paths(workspace: &Path) -> Vec<String> {
    crate::external_agents::artifacts::collect(workspace).unwrap()
}

#[test]
fn embedded_base64_is_rejected_before_or_after_decoding_when_oversized() {
    decode_bounded(STANDARD.encode(b"12345"), 4, "resource").expect_err("decoded limit");
    decode_bounded("AAAAAA==".repeat(2), 4, "media").expect_err("encoded limit");
    assert_eq!(
        decode_bounded(STANDARD.encode(b"1234"), 4, "resource").unwrap(),
        b"1234"
    );
}

#[tokio::test]
async fn embedded_document_is_saved_as_workspace_artifact() {
    let workspace =
        std::env::temp_dir().join(format!("knovum-agent-resource-{}", uuid::Uuid::new_v4()));
    tokio::fs::create_dir(&workspace).await.unwrap();
    let resource = EmbeddedResource::new(EmbeddedResourceResource::TextResourceContents(
        TextResourceContents::new("# Findings", "file:///report.md").mime_type("text/markdown"),
    ));
    let path = persist_resource(&workspace, resource, None).await.unwrap();
    assert_eq!(
        path.extension().and_then(|value| value.to_str()),
        Some("md")
    );
    assert!(path
        .file_name()
        .unwrap()
        .to_string_lossy()
        .contains("report"));
    assert_eq!(
        resource_stem("file:///final%20report.md"),
        Some("final report".into())
    );
    assert_eq!(
        tokio::fs::read_to_string(&path).await.unwrap(),
        "# Findings"
    );
    tokio::fs::remove_dir_all(workspace).await.unwrap();
}

#[tokio::test]
async fn repeated_media_chunks_are_distinct_occurrences_within_one_message() {
    use agent_client_protocol::schema::v1::{ContentChunk, ImageContent};

    let workspace =
        std::env::temp_dir().join(format!("agent-message-media-{}", uuid::Uuid::new_v4()));
    tokio::fs::create_dir(&workspace).await.unwrap();
    for _ in 0..2 {
        receive(
            SessionUpdate::AgentMessageChunk(
                ContentChunk::new(ContentBlock::Image(
                    ImageContent::new(STANDARD.encode(b"same"), "image/png")
                        .uri("https://example.com/cover.png"),
                ))
                .message_id("message-1"),
            ),
            &workspace,
        )
        .await
        .unwrap();
    }
    let artifacts = crate::external_agents::artifacts::collect(&workspace).unwrap();
    assert_eq!(artifacts.len(), 2);
    assert!(artifacts.iter().all(|artifact| artifact.contains("cover")));
    tokio::fs::remove_dir_all(workspace).await.unwrap();
}

#[tokio::test]
async fn resource_names_fit_filesystem_components_without_truncating_identity() {
    let workspace =
        std::env::temp_dir().join(format!("agent-resource-name-{}", uuid::Uuid::new_v4()));
    tokio::fs::create_dir(&workspace).await.unwrap();
    for stem in ["a".repeat(300), "长文件名称".repeat(70)] {
        let uri = tauri::Url::from_file_path(workspace.join(format!("{stem}.png")))
            .unwrap()
            .to_string();
        let stem = resource_stem(&uri).unwrap();
        let identity = message_occurrence_identity(Some("message"));
        let path = persist_media(
            &workspace,
            STANDARD.encode(b"image"),
            "image/png",
            &stem,
            Some(&identity),
        )
        .await
        .unwrap();
        let name = path.file_name().unwrap().to_str().unwrap();
        assert!(name.len() <= 255);
        assert!(name.starts_with(&identity));
        assert!(stem.len() <= 64);
    }
    tokio::fs::remove_dir_all(workspace).await.unwrap();
}

#[tokio::test]
async fn tool_outputs_replace_the_same_slot_and_distinct_slots_remain_distinct() {
    let workspace =
        std::env::temp_dir().join(format!("agent-tool-output-{}", uuid::Uuid::new_v4()));
    tokio::fs::create_dir(&workspace).await.unwrap();
    let content = vec![
        ToolCallContent::from(ContentBlock::Image(
            agent_client_protocol::schema::v1::ImageContent::new(
                STANDARD.encode(b"first"),
                "image/png",
            ),
        )),
        ToolCallContent::from(ContentBlock::Audio(
            agent_client_protocol::schema::v1::AudioContent::new(
                STANDARD.encode(b"obsolete"),
                "audio/wav",
            ),
        )),
    ];
    persist_tool_content(&workspace, "tool-1", &content)
        .await
        .unwrap();
    persist_tool_content(&workspace, "tool-1", &content)
        .await
        .unwrap();
    persist_tool_content(&workspace, "tool-2", &content[..1])
        .await
        .unwrap();
    let changed = vec![ToolCallContent::from(ContentBlock::Image(
        agent_client_protocol::schema::v1::ImageContent::new(
            STANDARD.encode(b"latest"),
            "image/png",
        ),
    ))];
    persist_tool_content(&workspace, "tool-1", &changed)
        .await
        .unwrap();
    let artifacts = crate::external_agents::artifacts::collect(&workspace).unwrap();
    assert_eq!(artifacts.len(), 2);
    let mut values = Vec::new();
    for artifact in artifacts {
        let path =
            crate::external_agents::artifacts::resolve_collected(&workspace, &artifact).unwrap();
        values.push(tokio::fs::read(path).await.unwrap());
    }
    assert!(values.contains(&b"first".to_vec()));
    assert!(values.contains(&b"latest".to_vec()));
    assert!(!values.contains(&b"obsolete".to_vec()));
    tokio::fs::remove_dir_all(workspace).await.unwrap();
}

#[tokio::test]
async fn a_tool_link_is_shown_never_fetched_or_published() {
    // codex-acp reports the image it looked at as `resource_link { uri: <bare path> }`; a link in
    // a tool call names a file that lives elsewhere, so it can neither fail the turn nor become
    // an output.
    let workspace = std::env::temp_dir().join(format!("agent-tool-links-{}", uuid::Uuid::new_v4()));
    tokio::fs::create_dir(&workspace).await.unwrap();
    let written = workspace.join("summary.md");
    tokio::fs::write(&written, "summary").await.unwrap();
    let links = [
        "/Users/someone/Desktop/deck.pptx".to_owned(),
        "slides/page-1.png".to_owned(),
        "https://example.com/page".to_owned(),
        tauri::Url::from_file_path(&written).unwrap().to_string(),
    ]
    .map(|uri| ToolCallContent::from(ContentBlock::ResourceLink(ResourceLink::new("Link", uri))));

    let update = receive(
        SessionUpdate::ToolCall(
            agent_client_protocol::schema::v1::ToolCall::new("view", "View Image")
                .content(links.to_vec()),
        ),
        &workspace,
    )
    .await
    .unwrap();

    assert!(matches!(update, Some(AgentSessionUpdate::ToolCall { .. })));
    assert_eq!(collected_paths(&workspace), vec!["summary.md"]);
    tokio::fs::remove_dir_all(workspace).await.unwrap();
}

#[tokio::test]
async fn an_agents_reasoning_is_neither_shown_nor_recorded() {
    use agent_client_protocol::schema::v1::{ContentChunk, TextContent};

    let update = receive(
        SessionUpdate::AgentThoughtChunk(ContentChunk::new(ContentBlock::Text(TextContent::new(
            "private",
        )))),
        &std::env::temp_dir(),
    )
    .await
    .unwrap();
    assert!(update.is_none());
}
