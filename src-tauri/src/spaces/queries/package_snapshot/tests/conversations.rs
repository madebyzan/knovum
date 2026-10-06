#[tokio::test]
async fn exports_current_conversation_without_a_product_mode() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    crate::spaces::db::migrate(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)")
        .execute(&pool).await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    sqlx::query(
        "INSERT INTO spaces_conversations(id,space_id,title,active_branch_id,created_at,updated_at)
        VALUES('conversation','space','Notes','conversation',1,1)",
    )
    .execute(&mut *tx)
    .await
    .unwrap();
    let (conversations, messages) = super::load(&mut tx, "space").await.unwrap();
    assert!(messages.is_empty());
    assert_eq!(conversations.len(), 1);
    let conversation = &conversations[0];
    assert!(conversation.get("mode").is_none());
    assert_eq!(conversation["activeBranchId"], "conversation");
    assert_eq!(conversation["id"], "conversation");
}
