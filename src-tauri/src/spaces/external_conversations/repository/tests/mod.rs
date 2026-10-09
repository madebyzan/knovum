use super::*;

#[tokio::test]
async fn turn_links_external_session_to_shared_conversation_history() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    db::migrate(&pool).await.unwrap();
    sqlx::raw_sql("INSERT INTO spaces(id,kind,title,created_at,updated_at)
        VALUES('space','space','Space',1,1);
        INSERT INTO spaces_conversations(id,space_id,title,active_branch_id,created_at,updated_at)
        VALUES('history','space','Conversation','branch',1,1);
        INSERT INTO spaces_runs(id,scope_kind,home_space_id,principal_kind,principal_id,kind,state,policy_snapshot,input_snapshot,budget,created_at,updated_at)
        VALUES('run','space','space','external_agent','session','external_agent','running','{}','{}','{}',1,1);
        INSERT INTO spaces_external_agent_conversations(id,run_id,space_id,conversation_id,branch_id,runtime_kind,title,state,created_at,updated_at)
        VALUES('session','run','space','history','branch','codex','Agent','connected',1,1);")
        .execute(&pool).await.unwrap();

    let turn_id = start_turn(&pool, "session", "Question", &[]).await.unwrap();
    let link: (String, String, String, String) = sqlx::query_as(
        "SELECT t.conversation_id,m.conversation_id,m.branch_id,m.role
         FROM spaces_external_agent_turns t
         JOIN spaces_conversation_messages m ON m.id=t.message_id WHERE t.id=$1",
    )
    .bind(&turn_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        link,
        (
            "session".into(),
            "history".into(),
            "branch".into(),
            "assistant".into()
        )
    );
    let question: String = sqlx::query_scalar(
        "SELECT json_extract(content,'$.parts[0].text') FROM spaces_conversation_messages
         WHERE conversation_id='history' AND role='user'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(question, "Question");
    let updated: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM spaces_external_agent_conversations s
         JOIN spaces_conversations c ON c.id=s.conversation_id
         WHERE s.id='session' AND s.updated_at>1 AND c.updated_at=s.updated_at",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(updated, 1);
}
