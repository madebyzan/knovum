use super::{archive, ensure_conversation, list_conversations, unarchive};
use crate::spaces::assistant::AssistantMode;

#[tokio::test]
async fn archive_and_unarchive_preserve_conversation_identity_and_content() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("database");
    crate::spaces::db::migrate(&pool).await.expect("schema");
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)")
        .execute(&pool).await.expect("space");
    let mut tx = pool.begin().await.expect("transaction");
    ensure_conversation(
        &mut tx,
        "conversation",
        "space",
        "Research",
        AssistantMode::Ask,
        1,
    )
    .await
    .expect("conversation");
    tx.commit().await.expect("commit");

    archive(&pool, "conversation").await.expect("archive");
    assert!(list_conversations(&pool, "space", None, false)
        .await
        .expect("active")
        .is_empty());
    let archived = list_conversations(&pool, "space", None, true)
        .await
        .expect("archived");
    assert_eq!(archived.len(), 1);
    assert_eq!(archived[0].title, "Research");
    assert_eq!(archived[0].branch_id.as_deref(), Some("conversation"));

    unarchive(&pool, "conversation").await.expect("unarchive");
    assert!(list_conversations(&pool, "space", None, true)
        .await
        .expect("archived")
        .is_empty());
    let active = list_conversations(&pool, "space", None, false)
        .await
        .expect("active");
    assert_eq!(active[0].id, "conversation");
    assert_eq!(active[0].title, "Research");
    assert!(archive(&pool, "missing")
        .await
        .unwrap_err()
        .contains("unavailable"));
    assert!(unarchive(&pool, "missing")
        .await
        .unwrap_err()
        .contains("unavailable"));
}
