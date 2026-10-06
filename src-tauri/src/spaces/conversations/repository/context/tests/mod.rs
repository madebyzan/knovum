use serde_json::json;

#[test]
fn history_carries_authoritative_output_and_attachment_node_ids() {
    let content = json!({
        "parts":[{"type":"text","text":"Created picture"}],
        "outputs":[{"nodeId":"published-picture","title":"Girl","kind":"image"}],
        "attachments":[{"source":"node","nodeId":"source-picture","title":"Source","kind":"image","thumbnail":null}],
    });
    let references = super::resource_references(serde_json::from_value(content).unwrap());
    assert!(references.contains("published-picture"));
    assert!(references.contains("source-picture"));
    assert!(!references.contains("thumbnail"));
    assert!(
        super::resource_references(serde_json::from_value(json!({"parts":[]})).unwrap()).is_empty()
    );
}

#[tokio::test]
async fn large_older_history_cannot_crowd_out_latest_request_and_output_identity() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    crate::spaces::db::migrate(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)")
        .execute(&pool).await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    crate::spaces::conversations::repository::ensure_conversation(
        &mut tx,
        "conversation",
        "space",
        "Conversation",
        1,
    )
    .await
    .unwrap();
    for index in 0..40 {
        let content = if index == 39 {
            json!({"parts":[{"type":"text","text":"remove the background of the previous picture"}],
                "outputs":[{"nodeId":"latest-picture","title":"Girl","kind":"image"}]})
        } else {
            json!({"parts":[{"type":"text","text":"older context ".repeat(2000)}]})
        };
        crate::spaces::conversations::repository::append_message_in_transaction(
            &mut tx,
            "conversation",
            "conversation",
            super::super::ConversationMessageRole::Assistant,
            &content,
            index + 2,
        )
        .await
        .unwrap();
    }
    tx.commit().await.unwrap();
    let history = super::build(&pool, "conversation", "conversation")
        .await
        .unwrap()
        .unwrap();
    assert!(history.contains("latest-picture"));
    assert!(history.contains("remove the background"));
    assert!(history.chars().count() <= super::MAX_CONTEXT_CHARACTERS);
}
