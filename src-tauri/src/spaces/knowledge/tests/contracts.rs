use super::support::database;

#[tokio::test]
async fn semantic_view_excludes_credentials_and_projections() {
    let pool = database().await;
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at,last_opened_at) VALUES ('space','space','Space',1,1,1)")
        .execute(&pool).await.expect("space");
    let rows = [
        (
            "note",
            "note",
            r#"{"kind":"note","title":"Note","body":"","tags":[]}"#,
        ),
        (
            "credential",
            "credential",
            r#"{"kind":"credential","title":"Secret","credentialKind":"api-key"}"#,
        ),
        (
            "text",
            "text",
            r#"{"kind":"text","title":"Text","tiptapContent":{"type":"doc","content":[]},"plainText":"Text"}"#,
        ),
    ];
    for (id, kind, data) in rows {
        sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) VALUES ($1,'space',$2,0,0,$3,1,1)")
            .bind(id).bind(kind).bind(data).execute(&pool).await.expect("node");
    }
    let ids = sqlx::query_scalar::<_, String>("SELECT id FROM spaces_semantic_nodes ORDER BY id")
        .fetch_all(&pool)
        .await
        .expect("semantic view");
    assert_eq!(ids, vec!["note".to_string()]);
}

#[test]
fn change_temp_ids_are_globally_unique() {
    let operations = serde_json::from_value::<Vec<super::super::change_commit::KnowledgeChange>>(serde_json::json!([
        {"op":"create-object","tempId":"same","objectTypeId":"system:claim","title":"Claim","status":"draft","properties":{"statement":"x"}},
        {"op":"create-relation","tempId":"same","sourceId":"same","targetId":"node","kind":"supports","rationale":""}
    ])).expect("knowledge changes");
    let error = super::super::change_commit::validate_structure(&operations)
        .expect_err("duplicate temp id must fail");
    assert!(error.contains("unique"));
}

#[tokio::test]
async fn change_failure_rolls_back_objects_created_earlier_in_transaction() {
    let pool = database().await;
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at,last_opened_at) VALUES ('space','space','Space',1,1,1)")
        .execute(&pool).await.expect("space");
    let operations = serde_json::from_value::<Vec<super::super::change_commit::KnowledgeChange>>(
        serde_json::json!([
            {
                "op":"create-object",
                "tempId":"claim",
                "objectTypeId":"system:claim",
                "title":"Claim",
                "status":"draft",
                "properties":{"statement":"A test claim"}
            },
            {
                "op":"add-provenance",
                "targetKind":"node",
                "targetId":"claim",
                "sourceNodeId":"missing-source",
                "locator":{"sourceRevision":"rev","location":{"kind":"pdf","page":1}},
                "sourceRevision":"rev",
                "quote":"evidence"
            }
        ]),
    )
    .expect("knowledge changes");

    let mut tx = pool.begin().await.expect("begin knowledge change");
    let error = super::super::change_commit::apply(&mut tx, "space", "agent", operations, 20)
        .await
        .expect_err("invalid provenance source must fail");
    assert!(error.contains("source") || error.contains("reference"));
    tx.rollback()
        .await
        .expect("rollback failed knowledge change");

    let count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM spaces_nodes WHERE space_id='space'")
            .fetch_one(&pool)
            .await
            .expect("node count");
    assert_eq!(count, 0);
}

#[tokio::test]
async fn grounded_scope_rejects_nonsemantic_nodes() {
    let pool = database().await;
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at,last_opened_at) VALUES ('space','space','Space',1,1,1)")
        .execute(&pool).await.expect("space");
    sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) VALUES ('secret','space','credential',0,0,'{\"kind\":\"credential\",\"title\":\"Secret\",\"credentialKind\":\"api-key\"}',1,1)")
        .execute(&pool).await.expect("credential");
    let mut tx = pool.begin().await.expect("begin");
    let error = super::super::grounding::validate_new_scope_nodes(
        &mut tx,
        "space",
        "selection",
        &["secret".to_string()],
    )
    .await
    .expect_err("credential must not enter grounded AI scope");
    assert_eq!(
        error,
        "Grounded AI scope contains a node that is unavailable in this Space"
    );
}

#[tokio::test]
async fn grounded_citation_requires_matching_chunk_revision() {
    let pool = database().await;
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at,last_opened_at) VALUES ('space','space','Space',1,1,1)")
        .execute(&pool).await.expect("space");
    sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) VALUES ('note','space','note',0,0,'{\"kind\":\"note\",\"title\":\"Note\",\"tiptapContent\":{\"type\":\"doc\",\"content\":[]},\"plainText\":\"evidence\"}',1,1)")
        .execute(&pool).await.expect("note");
    let citation = serde_json::json!([{
        "state": "current",
        "chunkId": "node:note",
        "nodeId": "note",
        "nodeTitle": "Note",
        "sourceRevision": "wrong-revision",
        "locator": {"sourceRevision":"wrong-revision","location":{"kind":"node"}},
        "quote": "evidence"
    }]);
    let mut tx = pool.begin().await.expect("begin");
    let contract = super::super::grounding::GroundedThreadContract {
        space_id: "space".to_string(),
        scope: "space".to_string(),
        node_ids: Vec::new(),
    };
    let error = super::super::grounding::validate_current_citations(&mut tx, &contract, &citation)
        .await
        .expect_err("citation revision must match canonical chunk");
    assert!(error.contains("source revision"));
}

#[tokio::test]
async fn all_spaces_scope_accepts_detached_external_citations() {
    let pool = database().await;
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at,last_opened_at) VALUES ('space','space','Space',1,1,1)")
        .execute(&pool).await.expect("space");
    let citation = serde_json::json!([{
        "state": "detached",
        "sourceSpaceTitle": "Research",
        "nodeTitle": "External",
        "sourceRevision": "node1:external",
        "locator": {"sourceRevision":"node1:external","location":{"kind":"node"}},
        "quote": "external evidence"
    }]);
    let contract = super::super::grounding::GroundedThreadContract {
        space_id: "space".to_string(),
        scope: "all-spaces".to_string(),
        node_ids: Vec::new(),
    };
    let mut tx = pool.begin().await.expect("begin");
    super::super::grounding::validate_current_citations(&mut tx, &contract, &citation)
        .await
        .expect("All-Spaces retrieval may persist detached external snapshots");
}
