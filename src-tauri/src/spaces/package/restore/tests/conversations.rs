use serde_json::{json, Value};

#[tokio::test]
async fn restores_mode_free_history_with_branch_and_parent_identity() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    crate::spaces::db::migrate(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)")
        .execute(&pool).await.unwrap();
    let mut value = json!({"space":{"id":"space","kind":"space","title":"Space",
        "lastOpenedAt":1,"createdAt":1,"updatedAt":1}});
    for field in [
        "structureNodes",
        "layouts",
        "relationStyles",
        "objectTypes",
        "nodes",
        "scenes",
        "relations",
        "derivations",
        "runs",
        "runSteps",
        "runAttempts",
        "runEvents",
        "externalAgentConversations",
        "externalAgentTurns",
        "voiceProfiles",
        "voiceRevisions",
        "speechProjects",
        "speechProjectRevisions",
        "speechSegments",
        "speechRenders",
        "speechSourceSeparations",
        "provenance",
        "knowledgeEvents",
        "chunks",
    ] {
        value[field] = json!([]);
    }
    value["conversations"] = json!([{"id":"conversation","title":"Notes","activeBranchId":"branch",
        "archivedAt":2,"createdAt":1,"updatedAt":2}]);
    value["conversationMessages"] = json!([
        {"id":"user","conversationId":"conversation","branchId":"branch","parentMessageId":null,
            "role":"user","content":{"parts":[{"type":"text","text":"Give a proposal"}]},"createdAt":1},
        {"id":"answer","conversationId":"conversation","branchId":"branch","parentMessageId":"user",
            "role":"assistant","content":{"parts":[{"type":"text","text":"Proposed steps"}]},"createdAt":2}
    ]);
    let request = serde_json::from_value(value).unwrap();
    let mut tx = pool.begin().await.unwrap();
    super::insert(&mut tx, &request).await.unwrap();
    tx.commit().await.unwrap();
    let conversation: (String, Option<i64>) = sqlx::query_as(
        "SELECT active_branch_id,archived_at FROM spaces_conversations WHERE id='conversation'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(conversation, ("branch".into(), Some(2)));
    let answer: (String, String, String) = sqlx::query_as(
        "SELECT parent_message_id,branch_id,content FROM spaces_conversation_messages WHERE id='answer'")
        .fetch_one(&pool).await.unwrap();
    assert_eq!(answer.0, "user");
    assert_eq!(answer.1, "branch");
    assert_eq!(
        serde_json::from_str::<Value>(&answer.2).unwrap()["parts"][0]["text"],
        "Proposed steps"
    );
}
