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
    sqlx::query("INSERT INTO spaces_edit_state(space_id,revision,updated_at) VALUES ('space',0,1)")
        .execute(&pool)
        .await
        .expect("edit state");
    let mut existing = crate::spaces::nodes::contracts::InsertNode {
        id: "existing".into(),
        kind: "note".into(),
        initial_x: 0.0,
        initial_y: 0.0,
        width: 256.0,
        height: 176.0,
        sizing_mode: crate::spaces::structure::SizingMode::Frame,
        appearance: None,
        file_subpath: None,
        data: serde_json::json!({
            "kind":"note", "title":"Existing", "plainText":"",
            "tiptapContent":{"type":"doc","content":[]}
        }),
        source_chunks: Vec::new(),
    };
    let mut setup = pool.begin().await.expect("begin canvas setup");
    crate::spaces::knowledge::validation::normalize_node(
        &mut setup,
        "space",
        "note",
        &mut existing.data,
    )
    .await
    .expect("normalize existing node");
    let (mut snapshot, revision) =
        crate::spaces::structure::repository::load_in_transaction(&mut setup, "space")
            .await
            .expect("load empty structure");
    crate::spaces::structure::creation::append_content(
        &mut snapshot,
        std::slice::from_ref(&existing),
        None,
    )
    .expect("project existing node");
    crate::spaces::nodes::insert_rows_with_actor(
        &mut setup,
        "space",
        std::slice::from_ref(&existing),
        crate::spaces::knowledge::events::EventActor::new("human", Some("user")),
        1,
    )
    .await
    .expect("insert existing node");
    crate::spaces::structure::repository::save_in_transaction(
        &mut setup,
        "space",
        revision,
        &mut snapshot,
    )
    .await
    .expect("save existing structure");
    setup.commit().await.expect("commit canvas setup");
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
    let actor = super::super::events::EventActor::new("external_agent", Some("agent"));
    let error = super::super::change_commit::apply(&mut tx, "space", None, actor, operations, 20)
        .await
        .expect_err("invalid provenance source must fail");
    assert!(error.message.contains("source") || error.message.contains("reference"));
    tx.rollback()
        .await
        .expect("rollback failed knowledge change");

    let count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM spaces_nodes WHERE space_id='space'")
            .fetch_one(&pool)
            .await
            .expect("node count");
    assert_eq!(count, 1);
    let mut inspection = pool.begin().await.expect("begin canvas inspection");
    let snapshot =
        crate::spaces::structure::repository::load_in_transaction(&mut inspection, "space")
            .await
            .expect("unchanged canvas after rollback");
    assert_eq!(snapshot.0.structure.len(), 1);
    inspection
        .rollback()
        .await
        .expect("finish canvas inspection");
}

#[tokio::test]
async fn change_commit_preserves_actor_identity_across_rows_and_events() {
    let pool = database().await;
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at,last_opened_at) VALUES ('space','space','Space',1,1,1)")
        .execute(&pool).await.expect("space");
    sqlx::query("INSERT INTO spaces_edit_state(space_id,revision,updated_at) VALUES ('space',0,1)")
        .execute(&pool)
        .await
        .expect("edit state");
    let parameters = serde_json::json!({"operations": [
        {
            "op":"create-object",
            "tempId":"source",
            "objectTypeId":"system:claim",
            "title":"Source",
            "status":"draft",
            "properties":{"statement":"Source claim"}
        },
        {
            "op":"create-object",
            "tempId":"target",
            "objectTypeId":"system:claim",
            "title":"Target",
            "status":"draft",
            "properties":{"statement":"Target claim"}
        },
        {
            "op":"create-relation",
            "tempId":"relation",
            "sourceId":"source",
            "targetId":"target",
            "kind":"supports",
            "rationale":"Evidence"
        },
        {
            "op":"add-provenance",
            "targetKind":"relation",
            "targetId":"relation",
            "sourceNodeId":"source",
            "locator":{"sourceRevision":"rev","location":{"kind":"node"}},
            "sourceRevision":"rev",
            "quote":"Source claim"
        }
    ]});

    let mut tx = pool.begin().await.expect("begin knowledge change");
    let commit = super::super::action::commit_changes(
        &mut tx,
        &crate::spaces::authorization::Principal {
            kind: crate::spaces::authorization::PrincipalKind::ExternalAgent,
            id: "external-session".into(),
        },
        None,
        "space",
        &parameters,
        20,
    )
    .await
    .expect("commit changes");
    tx.commit().await.expect("commit knowledge change");

    let relation_actor = sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT actor_kind,actor_id FROM spaces_relations WHERE space_id='space'",
    )
    .fetch_one(&pool)
    .await
    .expect("relation actor");
    assert_eq!(
        relation_actor,
        ("external_agent".into(), Some("external-session".into()))
    );

    let provenance_actor = sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT actor_kind,actor_id FROM spaces_provenance WHERE space_id='space'",
    )
    .fetch_one(&pool)
    .await
    .expect("provenance actor");
    assert_eq!(
        provenance_actor,
        ("external_agent".into(), Some("external-session".into()))
    );

    let event_actors = sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT DISTINCT actor_kind,actor_id FROM spaces_knowledge_events WHERE space_id='space' AND actor_id='external-session'",
    )
    .fetch_all(&pool)
    .await
    .expect("event actors");
    assert_eq!(
        event_actors,
        vec![("external_agent".into(), Some("external-session".into()))]
    );

    let projected_nodes = sqlx::query_scalar::<_, String>(
        "SELECT node_id FROM spaces_structure_nodes WHERE space_id='space' ORDER BY order_index",
    )
    .fetch_all(&pool)
    .await
    .expect("projected knowledge objects");
    assert_eq!(projected_nodes.len(), 2);
    let layout_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM spaces_structure_layouts WHERE space_id='space'",
    )
    .fetch_one(&pool)
    .await
    .expect("knowledge object layouts");
    assert_eq!(layout_count, 2);

    let resolved = commit
        .get("resolvedIds")
        .and_then(serde_json::Value::as_object)
        .expect("resolved object ids");
    let source_id = resolved
        .get("source")
        .and_then(serde_json::Value::as_str)
        .expect("source id");
    let target_id = resolved
        .get("target")
        .and_then(serde_json::Value::as_str)
        .expect("target id");
    let positions = sqlx::query_scalar::<_, f64>(
        "SELECT json_extract(l.data,'$.transform.x') FROM spaces_structure_nodes s \
         JOIN spaces_structure_layouts l ON l.space_id=s.space_id AND l.structure_node_id=s.id \
         WHERE s.node_id IN ($1,$2) ORDER BY json_extract(l.data,'$.transform.x')",
    )
    .bind(source_id)
    .bind(target_id)
    .fetch_all(&pool)
    .await
    .expect("created object positions");
    assert_eq!(positions, vec![0.0, 304.0]);
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
    let error =
        super::super::grounding::validate_conversation_citations(&mut tx, "space", &citation)
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
    let mut tx = pool.begin().await.expect("begin");
    super::super::grounding::validate_conversation_citations(&mut tx, "space", &citation)
        .await
        .expect("All-Spaces retrieval may persist detached external snapshots");
}

#[tokio::test]
async fn change_commit_resolves_cited_provenance_updates_relations_and_clusters_beside_source() {
    let pool = database().await;
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at,last_opened_at) VALUES ('space','space','Space',1,1,1)")
        .execute(&pool).await.expect("space");
    sqlx::query("INSERT INTO spaces_edit_state(space_id,revision,updated_at) VALUES ('space',0,1)")
        .execute(&pool)
        .await
        .expect("edit state");
    sqlx::query("INSERT INTO spaces_runs(id,scope_kind,home_space_id,principal_kind,principal_id,kind,state,policy_snapshot,input_snapshot,budget,created_at,updated_at) VALUES ('run-1','space','space','external_agent','agent-1','external_agent','running','{}','{}','{}',1,1)")
        .execute(&pool).await.expect("run");

    let mut source_node = crate::spaces::nodes::contracts::InsertNode {
        id: "source-doc".into(),
        kind: "note".into(),
        initial_x: 1000.0,
        initial_y: 500.0,
        width: 256.0,
        height: 176.0,
        sizing_mode: crate::spaces::structure::SizingMode::Frame,
        appearance: None,
        file_subpath: None,
        data: serde_json::json!({
            "kind":"note", "title":"Source Doc", "plainText":"Key finding in source",
            "tiptapContent":{"type":"doc","content":[]}
        }),
        source_chunks: Vec::new(),
    };
    let mut setup = pool.begin().await.expect("begin setup");
    crate::spaces::knowledge::validation::normalize_node(
        &mut setup,
        "space",
        "note",
        &mut source_node.data,
    )
    .await
    .expect("normalize");
    let source_revision = source_node.data["knowledgeRevision"]
        .as_str()
        .expect("revision")
        .to_owned();
    let (mut snapshot, revision) =
        crate::spaces::structure::repository::load_in_transaction(&mut setup, "space")
            .await
            .expect("load structure");
    crate::spaces::structure::creation::append_content(
        &mut snapshot,
        std::slice::from_ref(&source_node),
        None,
    )
    .expect("project source");
    crate::spaces::nodes::insert_rows_with_actor(
        &mut setup,
        "space",
        std::slice::from_ref(&source_node),
        crate::spaces::knowledge::events::EventActor::new("human", Some("user")),
        1,
    )
    .await
    .expect("insert source");
    crate::spaces::structure::repository::save_in_transaction(
        &mut setup,
        "space",
        revision,
        &mut snapshot,
    )
    .await
    .expect("save structure");
    let locator = crate::spaces::knowledge::indexing::locator(
        &source_revision,
        serde_json::json!({"kind":"node"}),
        "Key finding in source",
    );
    sqlx::query("INSERT INTO spaces_run_citations(run_id,ordinal,chunk_id,source_revision,node_id,node_title,locator,quote,created_at) VALUES('run-1',1,'chunk-1',$1,'source-doc','Source Doc',$2,'Key finding in source',2)")
        .bind(&source_revision)
        .bind(locator.to_string())
        .execute(&mut *setup)
        .await
        .expect("run citation");
    setup.commit().await.expect("commit setup");

    let principal = crate::spaces::authorization::Principal {
        kind: crate::spaces::authorization::PrincipalKind::ExternalAgent,
        id: "agent-1".into(),
    };
    let create_params = serde_json::json!({"operations": [
        {
            "op":"create-object",
            "tempId":"decision",
            "objectTypeId":"system:decision",
            "title":"Adopt Strategy",
            "status":"draft",
            "properties":{"decision":"Use hybrid search"}
        },
        {
            "op":"create-relation",
            "tempId":"rel",
            "sourceId":"decision",
            "targetId":"source-doc",
            "kind":"derives-from",
            "rationale":"Derived from source doc"
        },
        {
            "op":"add-cited-provenance",
            "targetKind":"node",
            "targetId":"decision",
            "citeId":"C1"
        }
    ]});
    let mut tx = pool.begin().await.expect("begin commit");
    let commit = super::super::action::commit_changes(
        &mut tx,
        &principal,
        Some("run-1"),
        "space",
        &create_params,
        20,
    )
    .await
    .expect("commit with cited provenance");
    tx.commit().await.expect("commit tx");

    let decision_id = commit["resolvedIds"]["decision"]
        .as_str()
        .expect("decision id");
    let relation_id = commit["resolvedIds"]["rel"].as_str().expect("relation id");
    let provenance = sqlx::query_as::<_, (String, String, String)>(
        "SELECT source_node_id, source_revision, quote FROM spaces_provenance WHERE target_id=$1",
    )
    .bind(decision_id)
    .fetch_one(&pool)
    .await
    .expect("provenance row");
    assert_eq!(
        provenance,
        (
            "source-doc".into(),
            source_revision,
            "Key finding in source".into()
        )
    );
    let decision_x = sqlx::query_scalar::<_, f64>(
        "SELECT json_extract(l.data,'$.transform.x') FROM spaces_structure_nodes s \
         JOIN spaces_structure_layouts l ON l.space_id=s.space_id AND l.structure_node_id=s.id \
         WHERE s.node_id=$1",
    )
    .bind(decision_id)
    .fetch_one(&pool)
    .await
    .expect("decision layout x");
    assert!(
        decision_x > 1000.0,
        "expected decision ({decision_x}) to be placed beside source-doc at x=1000"
    );

    let update_params = serde_json::json!({"operations": [
        {
            "op":"update-relation",
            "relationId": relation_id,
            "status":"contested",
            "rationale":"Needs benchmark confirmation",
            "confidence": 0.65
        }
    ]});
    let mut update_tx = pool.begin().await.expect("begin update relation");
    super::super::action::commit_changes(
        &mut update_tx,
        &principal,
        Some("run-1"),
        "space",
        &update_params,
        30,
    )
    .await
    .expect("update relation");
    update_tx.commit().await.expect("commit update relation");

    let updated = sqlx::query_as::<_, (String, String, Option<f64>)>(
        "SELECT status, rationale, confidence FROM spaces_relations WHERE id=$1",
    )
    .bind(relation_id)
    .fetch_one(&pool)
    .await
    .expect("updated relation");
    assert_eq!(
        updated,
        (
            "contested".into(),
            "Needs benchmark confirmation".into(),
            Some(0.65)
        )
    );

    let invalidate_params = serde_json::json!({"operations": [
        {
            "op":"update-relation",
            "relationId": relation_id,
            "status":"invalidated"
        }
    ]});
    let mut invalidate_tx = pool.begin().await.expect("begin invalid relation update");
    let invalid_error = super::super::action::commit_changes(
        &mut invalidate_tx,
        &principal,
        Some("run-1"),
        "space",
        &invalidate_params,
        40,
    )
    .await
    .expect_err("update-relation must reject invalidated status");
    assert!(invalid_error.message.contains("Invalidated"));
    invalidate_tx
        .rollback()
        .await
        .expect("rollback invalid relation update");
}
