use super::*;
use serde_json::json;

async fn fixture() -> sqlx::SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    db::migrate(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)")
            .execute(&pool).await.unwrap();
    let mut tx = db::begin_write(&pool).await.unwrap();
    structure::initialize_empty_in_transaction(&mut tx, "space")
        .await
        .unwrap();
    tx.commit().await.unwrap();
    sqlx::query("INSERT INTO spaces_assistant_conversations(id,space_id,title,mode,created_at,updated_at,active_branch_id) VALUES('conversation','space','Agent','ask',1,1,'conversation')")
            .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces_runs(id,home_space_id,principal_kind,principal_id,kind,state,policy_snapshot,input_snapshot,budget,created_at,updated_at) VALUES('run','space','external_agent','session','external_agent','running','{}','{\"attachments\":[]}','{}',1,1)")
            .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces_external_agent_conversations(id,run_id,space_id,runtime_kind,title,state,created_at,updated_at,assistant_conversation_id,branch_id) VALUES('session','run','space','codex','Agent','connected',1,1,'conversation','conversation')")
            .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces_assistant_messages(id,conversation_id,branch_id,role,content,created_at) VALUES('reply','conversation','conversation','assistant','{\"parts\":[{\"type\":\"text\",\"text\":\"\"}],\"runId\":\"run\"}',1)")
            .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces_external_agent_turns(id,conversation_id,state,created_at,completed_at,assistant_message_id) VALUES('turn','session','completed',1,2,'reply')")
            .execute(&pool).await.unwrap();
    pool
}

fn output(id: &str, kind: &str) -> InsertNode {
    InsertNode {
        id: id.into(),
        kind: kind.into(),
        initial_x: 0.0,
        initial_y: 0.0,
        width: 260.0,
        height: 184.0,
        sizing_mode: structure::SizingMode::Content,
        appearance: None,
        file_subpath: None,
        data: json!({"kind":kind,"title":"Report","text":"Findings"}),
        source_chunks: Vec::new(),
    }
}

#[tokio::test]
async fn publishes_nodes_message_and_run_in_one_commit() {
    let pool = fixture().await;
    commit(
        &pool,
        "session",
        "run",
        "space",
        &[output("report", "markdown")],
        None,
    )
    .await
    .unwrap();
    let result: (String, String, String, i64) = sqlx::query_as(
        "SELECT r.state,m.content,n.kind,e.revision FROM spaces_runs r
             JOIN spaces_assistant_messages m ON m.id='reply'
             JOIN spaces_nodes n ON n.id='report'
             JOIN spaces_edit_state e ON e.space_id='space' WHERE r.id='run'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(result.0, "succeeded");
    assert_eq!(result.2, "markdown");
    assert_eq!(result.3, 1);
    let message: Value = serde_json::from_str(&result.1).unwrap();
    assert_eq!(message["outputs"][0]["nodeId"], "report");
}

#[tokio::test]
async fn invalid_output_rolls_back_all_publication_state() {
    let pool = fixture().await;
    assert!(commit(
        &pool,
        "session",
        "run",
        "space",
        &[output("good", "markdown"), output("bad", "invalid")],
        None
    )
    .await
    .is_err());
    let node_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces_nodes")
        .fetch_one(&pool)
        .await
        .unwrap();
    let state: String = sqlx::query_scalar("SELECT state FROM spaces_runs WHERE id='run'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(node_count, 0);
    assert_eq!(state, "running");
}

#[tokio::test]
async fn existing_agent_created_node_is_attached_without_duplicate_import() {
    let pool = fixture().await;
    let mut tx = db::begin_write(&pool).await.unwrap();
    let mut node = output("mcp-note", "markdown");
    knowledge::contracts::normalize_node(&mut tx, "space", &node.kind, &mut node.data)
        .await
        .unwrap();
    let (mut snapshot, revision) = structure::repository::load_in_transaction(&mut tx, "space")
        .await
        .unwrap();
    structure::creation::append_content(&mut snapshot, std::slice::from_ref(&node), None).unwrap();
    nodes::insert_rows_with_actor(
        &mut tx,
        "space",
        std::slice::from_ref(&node),
        knowledge::events::EventActor::new("agent", Some("session")),
        2,
        None,
    )
    .await
    .unwrap();
    structure::repository::save_in_transaction(&mut tx, "space", revision, &snapshot)
        .await
        .unwrap();
    tx.commit().await.unwrap();

    assert!(super::super::repository::has_turn_output(&pool, "turn")
        .await
        .unwrap());
    commit(&pool, "session", "run", "space", &[], None)
        .await
        .unwrap();
    let content: String =
        sqlx::query_scalar("SELECT content FROM spaces_assistant_messages WHERE id='reply'")
            .fetch_one(&pool)
            .await
            .unwrap();
    let content: Value = serde_json::from_str(&content).unwrap();
    assert_eq!(content["outputs"][0]["nodeId"], "mcp-note");
    let node_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces_nodes")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(node_count, 1);
}

#[tokio::test]
async fn context_attachment_does_not_assert_derivation() {
    let pool = fixture().await;
    let mut tx = db::begin_write(&pool).await.unwrap();
    nodes::insert_many_in_transaction(
        &mut tx,
        nodes::contracts::InsertNodesRequest {
            space_id: "space".into(),
            mutation_id: "source-create".into(),
            expected_edit_revision: 0,
            target_parent_id: None,
            nodes: vec![output("source", "markdown")],
        },
        None,
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    sqlx::query("UPDATE spaces_runs SET input_snapshot=$1 WHERE id='run'")
        .bind(json!({"attachments":[{"source":"node","nodeId":"source"}]}).to_string())
        .execute(&pool)
        .await
        .unwrap();
    commit(
        &pool,
        "session",
        "run",
        "space",
        &[output("derived", "markdown")],
        None,
    )
    .await
    .unwrap();
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM spaces_relations WHERE space_id='space'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn cancel_request_prevents_publication() {
    let pool = fixture().await;
    sqlx::query("UPDATE spaces_runs SET cancel_requested_at=3 WHERE id='run'")
        .execute(&pool)
        .await
        .unwrap();
    assert!(commit(
        &pool,
        "session",
        "run",
        "space",
        &[output("late", "markdown")],
        None
    )
    .await
    .is_err());
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces_nodes")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn disconnected_session_keeps_completed_turn_outputs_retryable() {
    let pool = fixture().await;
    super::super::runtime::lifecycle::mark_disconnected(
        &pool,
        "session",
        "run",
        "ACP finish failed",
    )
    .await
    .unwrap();
    commit(
        &pool,
        "session",
        "run",
        "space",
        &[output("report", "markdown")],
        None,
    )
    .await
    .unwrap();
    let state: String = sqlx::query_scalar("SELECT state FROM spaces_runs WHERE id='run'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(state, "succeeded");
}
#[tokio::test]
async fn publication_failure_is_retryable_and_receipts_are_released_atomically() {
    let pool = fixture().await;
    sqlx::query("UPDATE spaces_runs SET state='failed',completed_at=3 WHERE id='run'")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("UPDATE spaces_external_agent_conversations SET output_publication_state='failed',output_publication_error='Disk full' WHERE id='session'")
            .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces_external_output_staging(run_id,path,role,created_at) VALUES('run','/managed/report','asset',1)")
            .execute(&pool).await.unwrap();
    commit(
        &pool,
        "session",
        "run",
        "space",
        &[output("report", "markdown")],
        None,
    )
    .await
    .unwrap();
    let state: (String, String, i64) = sqlx::query_as(
            "SELECT r.state,c.output_publication_state,(SELECT COUNT(*) FROM spaces_external_output_staging) FROM spaces_runs r JOIN spaces_external_agent_conversations c ON c.run_id=r.id WHERE r.id='run'")
            .fetch_one(&pool).await.unwrap();
    assert_eq!(state, ("succeeded".into(), "committed".into(), 0));
    assert!(commit(
        &pool,
        "session",
        "run",
        "space",
        &[output("duplicate", "markdown")],
        None
    )
    .await
    .is_err());
    let actor: String = sqlx::query_scalar("SELECT actor_id FROM spaces_knowledge_events WHERE entity_id='report' AND event_kind='created'")
            .fetch_one(&pool).await.unwrap();
    assert_eq!(actor, "session");
}

#[tokio::test]
async fn unfinished_children_block_publication_and_failed_children_are_visible() {
    let pool = fixture().await;
    sqlx::query("INSERT INTO spaces_runs(id,home_space_id,parent_run_id,principal_kind,principal_id,kind,state,policy_snapshot,input_snapshot,budget,created_at,updated_at) VALUES('child','space','run','external_agent','session','action','running','{}','{}','{}',1,1)")
            .execute(&pool).await.unwrap();
    assert!(commit(
        &pool,
        "session",
        "run",
        "space",
        &[output("too-early", "markdown")],
        None
    )
    .await
    .is_err());
    sqlx::query("UPDATE spaces_runs SET state='failed',completed_at=3 WHERE id='child'")
        .execute(&pool)
        .await
        .unwrap();
    commit(
        &pool,
        "session",
        "run",
        "space",
        &[output("report", "markdown")],
        None,
    )
    .await
    .unwrap();
    let state: String = sqlx::query_scalar("SELECT state FROM spaces_runs WHERE id='run'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(state, "completed_with_errors");
}

#[tokio::test]
async fn cancelling_run_immediately_cancels_all_descendants() {
    let pool = fixture().await;
    for (id, parent) in [("child", "run"), ("grandchild", "child")] {
        sqlx::query("INSERT INTO spaces_runs(id,home_space_id,parent_run_id,principal_kind,principal_id,kind,state,policy_snapshot,input_snapshot,budget,created_at,updated_at) VALUES($1,'space',$2,'external_agent','session','action','running','{}','{}','{}',1,1)")
                .bind(id).bind(parent).execute(&pool).await.unwrap();
    }
    crate::spaces::runs::request_cancellation(&pool, "run")
        .await
        .unwrap();
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces_runs WHERE cancel_requested_at IS NOT NULL AND execution_epoch=1")
            .fetch_one(&pool).await.unwrap();
    assert_eq!(count, 3);
    assert!(commit(&pool, "session", "run", "space", &[], None)
        .await
        .is_err());
}
