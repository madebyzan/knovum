use super::*;

async fn fixture() -> sqlx::SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    db::migrate(&pool).await.unwrap();
    for statement in [
        "INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)",
        "INSERT INTO spaces_conversations(id,space_id,title,created_at,updated_at,active_branch_id) VALUES('conversation','space','Agent',1,1,'conversation')",
        "INSERT INTO spaces_runs(id,home_space_id,principal_kind,principal_id,kind,state,policy_snapshot,input_snapshot,budget,created_at,updated_at) VALUES('run','space','external_agent','session','external_agent','queued','{}','{}','{}',1,1)",
        "INSERT INTO spaces_external_agent_conversations(id,run_id,space_id,runtime_kind,title,state,created_at,updated_at,conversation_id,branch_id) VALUES('session','run','space','geminiCli','Agent','connecting',1,1,'conversation','conversation')",
        "INSERT INTO spaces_conversation_messages(id,conversation_id,branch_id,role,content,created_at) VALUES('reply','conversation','conversation','assistant','{}',1)",
        "INSERT INTO spaces_external_agent_turns(id,conversation_id,state,created_at,message_id) VALUES('turn','session','running',1,'reply')",
    ] {
        sqlx::query(statement).execute(&pool).await.unwrap();
    }
    pool
}

fn session() -> crate::external_agents::conversation::ConversationSession {
    let (_settled, completion) = tokio::sync::oneshot::channel();
    crate::external_agents::conversation::ConversationSession {
        runtime_session_id: "runtime".into(),
        capabilities: serde_json::json!({}),
        completion,
    }
}

async fn states(pool: &sqlx::SqlitePool) -> (String, String, String) {
    sqlx::query_as("SELECT c.state,r.state,t.state FROM spaces_external_agent_conversations c
        JOIN spaces_runs r ON r.id=c.run_id JOIN spaces_external_agent_turns t ON t.conversation_id=c.id WHERE c.id='session'")
        .fetch_one(pool).await.unwrap()
}

#[tokio::test]
async fn connection_requires_current_startup_authority() {
    let pool = fixture().await;
    mark_connected(&pool, "session", "run", &session())
        .await
        .unwrap();
    assert_eq!(
        states(&pool).await,
        ("connected".into(), "running".into(), "running".into())
    );
    assert!(mark_connected(&pool, "session", "run", &session())
        .await
        .is_err());
    mark_disconnected(&pool, "session", "run", "lost connection")
        .await
        .unwrap();
    assert!(mark_connected(&pool, "session", "run", &session())
        .await
        .is_err());
}

#[tokio::test]
async fn disconnection_settles_all_active_records_once_and_preserves_the_first_error() {
    let pool = fixture().await;
    assert!(
        mark_disconnected(&pool, "session", "run", "transport closed")
            .await
            .unwrap()
    );
    assert!(!mark_disconnected(&pool, "session", "run", "later error")
        .await
        .unwrap());
    assert_eq!(
        states(&pool).await,
        ("disconnected".into(), "failed".into(), "failed".into())
    );
    let errors: (String, String, i64, i64) = sqlx::query_as(
        "SELECT c.error,t.error,r.completed_at,t.completed_at
        FROM spaces_external_agent_conversations c JOIN spaces_runs r ON r.id=c.run_id
        JOIN spaces_external_agent_turns t ON t.conversation_id=c.id WHERE c.id='session'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(errors.0, "transport closed");
    assert_eq!(errors.1, "transport closed");
    assert_eq!(errors.2, errors.3);
}

#[tokio::test]
async fn cancellation_and_committed_publication_cannot_be_overwritten_by_disconnection() {
    for statement in [
        "UPDATE spaces_runs SET cancel_requested_at=2 WHERE id='run'",
        "UPDATE spaces_external_agent_conversations SET state='closed',output_publication_state='committed' WHERE id='session'",
    ] {
        let pool = fixture().await;
        sqlx::query(statement).execute(&pool).await.unwrap();
        let before = states(&pool).await;
        assert!(!mark_disconnected(&pool, "session", "run", "transport closed").await.unwrap());
        assert_eq!(states(&pool).await, before);
        assert!(mark_connected(&pool, "session", "run", &session()).await.is_err());
    }
}

#[tokio::test]
async fn terminal_run_cannot_be_reactivated_by_a_late_connection() {
    let pool = fixture().await;
    sqlx::query("UPDATE spaces_runs SET state='failed',completed_at=2 WHERE id='run'")
        .execute(&pool)
        .await
        .unwrap();
    assert!(mark_connected(&pool, "session", "run", &session())
        .await
        .is_err());
    assert_eq!(
        states(&pool).await,
        ("connecting".into(), "failed".into(), "running".into())
    );
}

#[tokio::test]
async fn failure_to_settle_the_run_rolls_back_conversation_and_turn_changes() {
    let pool = fixture().await;
    sqlx::query("CREATE TRIGGER reject_run_update BEFORE UPDATE ON spaces_runs BEGIN SELECT RAISE(ABORT,'write rejected'); END")
        .execute(&pool).await.unwrap();
    assert!(
        mark_disconnected(&pool, "session", "run", "transport closed")
            .await
            .is_err()
    );
    assert_eq!(
        states(&pool).await,
        ("connecting".into(), "queued".into(), "running".into())
    );
}
