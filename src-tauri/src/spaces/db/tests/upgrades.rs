//! Each migration upgrades the schema the previous release shipped. A test here starts from that
//! schema, carries representative rows through the migration, and checks that the upgraded library
//! is the one a fresh install creates, with the rows it had.

use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

/// A library exactly as `version` left it.
async fn shipped(version: i64) -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("database");
    super::super::MIGRATOR
        .run_to(version, &pool)
        .await
        .expect("shipped schema");
    pool
}

async fn schema(pool: &SqlitePool) -> Vec<String> {
    sqlx::query_scalar("SELECT sql FROM sqlite_master WHERE sql IS NOT NULL ORDER BY name")
        .fetch_all(pool)
        .await
        .expect("schema")
}

/// 0.0.1 recorded the sessions of the removed Create with Agent dialog. Upgrading settles the Run
/// a crash left open — nothing can finish it any more — and keeps every Run, while the session
/// table leaves with the feature.
#[tokio::test]
async fn upgrading_0_0_1_settles_and_retires_agent_artifact_sessions() {
    let pool = shipped(1).await;
    sqlx::query(
        "INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('s','space','S',1,1)",
    )
    .execute(&pool)
    .await
    .expect("space");
    for (run, state, completed_at) in [
        ("interrupted", "running", None),
        ("finished", "succeeded", Some(5_i64)),
        ("unrelated", "running", None),
    ] {
        sqlx::query(
            "INSERT INTO spaces_runs(id,scope_kind,home_space_id,principal_kind,principal_id,kind, \
             state,state_version,policy_snapshot,input_snapshot,budget,created_at,updated_at, \
             completed_at) \
             VALUES($1,'space','s','external_agent',$1,'external_agent',$2,1,'{}','{}','{}',1,1,$3)",
        )
        .bind(run)
        .bind(state)
        .bind(completed_at)
        .execute(&pool)
        .await
        .expect("run");
    }
    for (session, state) in [("interrupted", "running"), ("finished", "completed")] {
        sqlx::query(
            "INSERT INTO spaces_agent_artifact_sessions(id,run_id,space_id,runtime_kind,state, \
             created_at,updated_at) VALUES($1,$1,'s','codex',$2,1,1)",
        )
        .bind(session)
        .bind(state)
        .execute(&pool)
        .await
        .expect("session");
    }

    super::super::migrate(&pool).await.expect("upgrade");

    assert_eq!(schema(&pool).await, schema(&super::migrated().await).await);
    let runs: Vec<(String, String, i64, Option<i64>)> =
        sqlx::query_as("SELECT id,state,state_version,completed_at FROM spaces_runs ORDER BY id")
            .fetch_all(&pool)
            .await
            .expect("runs");
    let settled: Vec<_> = runs
        .iter()
        .map(|(id, state, version, completed_at)| {
            (
                id.as_str(),
                state.as_str(),
                *version,
                completed_at.is_some_and(|at| at > 1),
            )
        })
        .collect();
    assert_eq!(
        settled,
        [
            ("finished", "succeeded", 1, true),
            ("interrupted", "failed", 2, true),
            ("unrelated", "running", 1, false),
        ]
    );
}

/// 0.0.3 kept a picture of an attached image node in the message. A node attachment is a
/// reference drawn from the node itself now, so upgrading takes the picture out of every node
/// attachment, in place and in order, and leaves an uploaded file's — the only copy it has.
#[tokio::test]
async fn upgrading_0_0_3_draws_node_attachments_from_their_nodes() {
    let pool = shipped(2).await;
    for statement in [
        "INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('s','space','S',1,1)",
        "INSERT INTO spaces_conversations(id,space_id,title,created_at,updated_at) \
         VALUES('c','s','C',1,1)",
    ] {
        sqlx::query(statement)
            .execute(&pool)
            .await
            .expect("fixture");
    }
    let sent = serde_json::json!({"parts": [], "citations": [], "attachments": [
        {"source": "node", "nodeId": "deck", "title": "Deck", "kind": "file", "thumbnail": null},
        {"source": "file", "title": "dog.png", "kind": "image", "thumbnail": "data:image/png;base64,AA"},
        {"source": "node", "nodeId": "dog", "title": "Dog", "kind": "image", "thumbnail": "data:image/png;base64,BB"},
    ]});
    let plain = serde_json::json!({"parts": [{"type": "text", "text": "hi"}], "citations": []});
    for (id, content) in [("sent", &sent), ("plain", &plain)] {
        sqlx::query(
            "INSERT INTO spaces_conversation_messages(id,conversation_id,branch_id,role,content, \
             created_at) VALUES($1,'c','c','user',$2,1)",
        )
        .bind(id)
        .bind(content.to_string())
        .execute(&pool)
        .await
        .expect("message");
    }

    super::super::migrate(&pool).await.expect("upgrade");

    assert_eq!(schema(&pool).await, schema(&super::migrated().await).await);
    let content = |id: &'static str| {
        let pool = pool.clone();
        async move {
            let text: String =
                sqlx::query_scalar("SELECT content FROM spaces_conversation_messages WHERE id=$1")
                    .bind(id)
                    .fetch_one(&pool)
                    .await
                    .expect("content");
            serde_json::from_str::<serde_json::Value>(&text).expect("json")
        }
    };
    assert_eq!(
        content("sent").await["attachments"],
        serde_json::json!([
            {"source": "node", "nodeId": "deck", "title": "Deck", "kind": "file"},
            {"source": "file", "title": "dog.png", "kind": "image", "thumbnail": "data:image/png;base64,AA"},
            {"source": "node", "nodeId": "dog", "title": "Dog", "kind": "image"},
        ])
    );
    assert_eq!(content("plain").await, plain);
}

/// 0.1.0 had no Knowledge Proposals and no answer verification. Upgrading adds both empty, and
/// every answer the library already holds can be verified at once.
#[tokio::test]
async fn upgrading_0_1_0_adds_proposals_and_answer_verification() {
    let pool = shipped(3).await;
    for statement in [
        "INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('s','space','S',1,1)",
        "INSERT INTO spaces_conversations(id,space_id,title,created_at,updated_at) VALUES('c','s','C',1,1)",
        "INSERT INTO spaces_conversation_messages(id,conversation_id,branch_id,role,content,created_at) \
         VALUES('answer','c','main','assistant','{\"parts\":[],\"citations\":[]}',1)",
    ] {
        sqlx::query(statement).execute(&pool).await.expect("shipped row");
    }
    super::super::migrate(&pool).await.expect("upgrade");
    assert_eq!(schema(&pool).await, schema(&super::migrated().await).await);
    sqlx::query(
        "INSERT INTO spaces_answer_verifications(message_id,space_id,answer_run_id,state,created_at) \
         VALUES('answer','s','run','pending',2)",
    )
    .execute(&pool)
    .await
    .expect("verification of a shipped answer");
}
