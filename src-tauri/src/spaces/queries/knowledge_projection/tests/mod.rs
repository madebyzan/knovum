use serde_json::{json, Value};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

async fn database() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    crate::spaces::db::migrate(&pool).await.unwrap();
    for id in ["space", "other"] {
        sqlx::query(
            "INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES($1,'space',$1,1,1)",
        )
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();
    }
    pool
}

async fn seed(pool: &SqlitePool, id: &str, kind: &str, space: &str) {
    let mut data = json!({"kind":kind,"title":id,"sourceRevision":"source-r1","assetPath":"/managed/source.png"});
    crate::spaces::knowledge::model::revision::apply(&mut data).unwrap();
    sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) VALUES($1,$2,$3,0,0,$4,1,1)")
        .bind(id).bind(space).bind(kind).bind(data.to_string()).execute(pool).await.unwrap();
}

#[tokio::test]
async fn inventory_includes_unindexed_resources_and_excludes_deleted_secrets_and_other_spaces() {
    let pool = database().await;
    for (id, kind) in [
        ("a", "image"),
        ("b", "video"),
        ("c", "file"),
        ("d", "text"),
        ("secret", "credential"),
        ("portal", "portal"),
        ("deleted", "image"),
    ] {
        seed(&pool, id, kind, "space").await;
    }
    seed(&pool, "other-node", "image", "other").await;
    // Node insertion creates metadata chunks. Remove these indexes to exercise
    // inventory independently of knowledge-index availability.
    sqlx::query("DELETE FROM spaces_knowledge_chunks WHERE node_id IN ('a','b')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("UPDATE spaces_nodes SET deleted_at=2 WHERE id='deleted'")
        .execute(&pool)
        .await
        .unwrap();
    let mut tx = pool.begin().await.unwrap();
    let page = super::list_contents(&mut tx, "space", 0, 2).await.unwrap();
    assert_eq!(page.space_id, "space");
    assert_eq!(page.total_nodes, 4);
    assert_eq!(
        page.nodes
            .iter()
            .map(|node| node.node_id.as_str())
            .collect::<Vec<_>>(),
        ["a", "b"]
    );
    assert!(page.nodes.iter().all(|node| !node.indexed));
    assert_eq!(page.nodes[0].source_revision.as_deref(), Some("source-r1"));
    assert_ne!(
        page.nodes[0].knowledge_revision,
        page.nodes[0].source_revision
    );
    let page = super::list_contents(&mut tx, "space", page.next_offset.unwrap(), 2)
        .await
        .unwrap();
    assert_eq!(
        page.nodes
            .iter()
            .map(|node| node.node_id.as_str())
            .collect::<Vec<_>>(),
        ["c", "d"]
    );
    assert_eq!(page.next_offset, None);
    assert!(page.nodes[0].indexed);
    assert!(!page.nodes[1].indexed);
    assert!(super::list_contents(&mut tx, "missing", 0, 40)
        .await
        .is_err());
    assert!(super::list_contents(&mut tx, "space", -1, 40)
        .await
        .is_err());
    assert!(super::list_contents(&mut tx, "space", 0, 101)
        .await
        .is_err());
    for id in ["deleted", "secret", "portal", "other-node"] {
        assert!(
            super::load_node(&mut tx, "space", id).await.is_err(),
            "{id}"
        );
    }
    let node = super::load_node(&mut tx, "space", "a").await.unwrap();
    assert_eq!(node["data"]["assetPath"], "/managed/source.png");
    assert_eq!(node["chunks"], json!([]));
    let (rendered_revision, _) = crate::spaces::derivation::input_revision::resolve(
        "image",
        &node["data"],
        crate::spaces::derivation::input_revision::RevisionBasis::Rendered,
    )
    .unwrap();
    assert_eq!(node["assetRevision"], rendered_revision);
    assert_ne!(node["assetRevision"], node["knowledgeRevision"]);
    assert_ne!(node["assetRevision"], node["sourceRevision"]);
}

#[tokio::test]
async fn search_actions_and_mcp_share_chinese_search_and_revision_aware_matches() {
    let pool = database().await;
    let data = json!({"kind":"note","title":"Space evidence","plainText":"画布有图片资料","tiptapContent":{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"画布有图片资料"}]}]}});
    let mut normalized = data;
    normalized["knowledgeRevision"] = Value::String(
        crate::spaces::knowledge::model::revision::authoritative(&normalized)
            .unwrap()
            .unwrap(),
    );
    sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) VALUES('note','space','note',0,0,$1,1,1)")
        .bind(normalized.to_string()).execute(&pool).await.unwrap();
    // Source-owned writes normally create these canonical chunks; exercise that same index contract.
    let mut tx = pool.begin().await.unwrap();
    let revision = normalized["knowledgeRevision"].as_str().unwrap();
    let locator = crate::spaces::knowledge::indexing::locator(
        revision,
        json!({"kind":"node"}),
        "画布有图片资料",
    );
    sqlx::query("INSERT INTO spaces_knowledge_chunks(id,space_id,node_id,origin,kind,source_revision,text,locator,metadata,created_at,updated_at) VALUES('chunk','space','note','derived','text',$1,'画布有图片资料',$2,'{}',1,1)")
        .bind(revision).bind(locator.to_string()).execute(&mut *tx).await.unwrap();
    let search_filter = |query: &str, limit: i64| super::SearchFilter {
        query: query.into(),
        limit,
        node_ids: None,
        roles: None,
        mode: super::SearchOutputMode::Passages,
    };
    let expected = super::search_knowledge(&mut tx, "space", &search_filter("图片", 20))
        .await
        .unwrap();
    assert_eq!(expected.as_array().unwrap().len(), 1);
    let principal = crate::spaces::authorization::Principal {
        kind: crate::spaces::authorization::PrincipalKind::User,
        id: "test".into(),
    };
    let action = crate::spaces::action_execution::execute_immediate(
        &mut tx,
        crate::spaces::action_execution::ImmediateActionRequest {
            principal: &principal,
            run_id: None,
            invocation_id: "search-action",
            space_id: "space",
            action_id: "search-knowledge",
            parameters: &json!({"query":"图片","limit":20}),
            now: 1,
        },
    )
    .await
    .unwrap();
    assert_eq!(action, expected);
    assert_eq!(action[0]["nodeId"], "note");
    assert_eq!(action[0]["sourceRevision"], revision);
    assert_eq!(action[0]["locator"], locator);
    assert!(
        super::search_knowledge(&mut tx, "other", &search_filter("图片", 20))
            .await
            .unwrap()
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(
        super::search_knowledge(&mut tx, "space", &search_filter(" ", 20))
            .await
            .is_err()
    );
    let direct = super::load_node(&mut tx, "space", "note").await.unwrap();
    let action = crate::spaces::action_execution::execute_immediate(
        &mut tx,
        crate::spaces::action_execution::ImmediateActionRequest {
            principal: &principal,
            run_id: None,
            invocation_id: "read-action",
            space_id: "space",
            action_id: "read-node",
            parameters: &json!({"nodeId":"note"}),
            now: 1,
        },
    )
    .await
    .unwrap();
    assert_eq!(direct, action);
    assert_eq!(direct["data"]["plainText"], "画布有图片资料");
    assert!(direct["data"].get("tiptapContent").is_none());
    sqlx::query("UPDATE spaces_nodes SET deleted_at=2 WHERE id='note'")
        .execute(&mut *tx)
        .await
        .unwrap();
    assert_eq!(
        super::list_contents(&mut tx, "space", 0, 40)
            .await
            .unwrap()
            .total_nodes,
        0
    );
    assert_eq!(
        super::search_knowledge(&mut tx, "space", &search_filter("图片", 20))
            .await
            .unwrap(),
        Value::Array(vec![])
    );
}

#[tokio::test]
async fn node_projection_compacts_raw_data_and_paginates_chunks_in_natural_order() {
    let pool = database().await;
    let mut video_data = json!({
        "kind": "video",
        "title": "Interview Clip",
        "summary": "Emma Watson discusses filming.",
        "durationSeconds": 180,
        "assetPath": "/managed/interview.mp4",
        "previewPath": "/managed/preview.jpg",
        "waveform": [0.1, 0.2, 0.3],
        "mediaKnowledge": {
            "transcript": {
                "id": "tr-1",
                "revision": "rev-1",
                "durationMs": 180000,
                "speakerTracks": [
                    {"id": "spk-1", "label": "Host"},
                    {"id": "spk-2", "label": "Guest"}
                ],
                "segments": [
                    {"startMs": 10000, "endMs": 20000, "speakerId": "spk-1", "text": "First"},
                    {"startMs": 20000, "endMs": 30000, "speakerId": "spk-2", "text": "Second"},
                    {"startMs": 30000, "endMs": 40000, "speakerId": "spk-1", "text": "Third"}
                ]
            }
        }
    });
    crate::spaces::knowledge::model::revision::apply(&mut video_data).unwrap();
    let revision = video_data["knowledgeRevision"].as_str().unwrap().to_owned();
    sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) VALUES('vid','space','video',0,0,$1,1,1)")
        .bind(video_data.to_string()).execute(&pool).await.unwrap();
    sqlx::query("DELETE FROM spaces_knowledge_chunks WHERE node_id='vid'")
        .execute(&pool)
        .await
        .unwrap();

    let chunk_rows = [
        (
            "c-seg-3",
            "segment",
            "Third line",
            json!({"kind":"media_time","startMs":30000,"endMs":40000}),
        ),
        (
            "c-seg-1",
            "segment",
            "First line",
            json!({"kind":"media_time","startMs":10000,"endMs":20000}),
        ),
        (
            "c-sum",
            "summary",
            "Overall summary",
            json!({"kind":"node"}),
        ),
        (
            "c-seg-2",
            "segment",
            "Second line",
            json!({"kind":"media_time","startMs":20000,"endMs":30000}),
        ),
        (
            "c-chap",
            "chapter",
            "Chapter 1",
            json!({"kind":"media_time","startMs":0,"endMs":60000}),
        ),
    ];
    for (id, role, text, location) in chunk_rows {
        let locator = crate::spaces::knowledge::indexing::locator(&revision, location, text);
        let metadata = json!({"role": role});
        sqlx::query("INSERT INTO spaces_knowledge_chunks(id,space_id,node_id,origin,kind,source_revision,text,locator,metadata,created_at,updated_at) VALUES($1,'space','vid','derived','text',$2,$3,$4,$5,1,1)")
            .bind(id)
            .bind(&revision)
            .bind(text)
            .bind(locator.to_string())
            .bind(metadata.to_string())
            .execute(&pool)
            .await
            .unwrap();
    }

    let mut tx = pool.begin().await.unwrap();
    let node = super::load_node(&mut tx, "space", "vid").await.unwrap();
    assert!(node["data"].get("previewPath").is_none());
    assert!(node["data"].get("waveform").is_none());
    assert!(node["data"]["mediaKnowledge"].get("transcript").is_none());
    assert_eq!(
        node["data"]["mediaKnowledge"]["transcriptMetadata"]["segmentCount"],
        3
    );
    assert_eq!(
        node["data"]["mediaKnowledge"]["transcriptMetadata"]["speakerCount"],
        2
    );
    assert_eq!(node["totalChunks"], 5);
    assert_eq!(node["nextChunksOffset"], Value::Null);
    let ordered_ids: Vec<&str> = node["chunks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|chunk| chunk["id"].as_str().unwrap())
        .collect();
    assert_eq!(
        ordered_ids,
        vec!["c-sum", "c-chap", "c-seg-1", "c-seg-2", "c-seg-3"]
    );

    let first_page = super::load_node_chunks(
        &mut tx,
        "space",
        "vid",
        super::ChunkReadFilter {
            role: None,
            offset: 0,
            limit: 2,
            around_chunk_id: None,
            context_chunks: None,
        },
    )
    .await
    .unwrap();
    assert_eq!(first_page["totalChunks"], 5);
    assert_eq!(first_page["nextOffset"], 2);
    assert_eq!(
        first_page["chunks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|chunk| chunk["id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["c-sum", "c-chap"]
    );

    let segment_page = super::load_node_chunks(
        &mut tx,
        "space",
        "vid",
        super::ChunkReadFilter {
            role: Some("segment".into()),
            offset: 1,
            limit: 2,
            around_chunk_id: None,
            context_chunks: None,
        },
    )
    .await
    .unwrap();
    assert_eq!(segment_page["totalChunks"], 3);
    assert_eq!(segment_page["nextOffset"], Value::Null);
    assert_eq!(
        segment_page["chunks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|chunk| chunk["id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["c-seg-2", "c-seg-3"]
    );

    let context_window = super::load_node_chunks(
        &mut tx,
        "space",
        "vid",
        super::ChunkReadFilter {
            role: None,
            offset: 0,
            limit: 50,
            around_chunk_id: Some("c-seg-2".into()),
            context_chunks: Some(1),
        },
    )
    .await
    .unwrap();
    assert_eq!(context_window["anchorChunkId"], "c-seg-2");
    assert_eq!(context_window["anchorOffset"], 3);
    assert_eq!(context_window["offset"], 2);
    assert_eq!(
        context_window["chunks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|chunk| chunk["id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["c-seg-1", "c-seg-2", "c-seg-3"]
    );
    assert!(super::load_node_chunks(
        &mut tx,
        "space",
        "vid",
        super::ChunkReadFilter {
            role: None,
            offset: 0,
            limit: 50,
            around_chunk_id: Some("missing-chunk".into()),
            context_chunks: Some(1),
        },
    )
    .await
    .is_err());

    let contents = super::list_contents(&mut tx, "space", 0, 20).await.unwrap();
    let video_entry = contents
        .nodes
        .iter()
        .find(|entry| entry.node_id == "vid")
        .unwrap();
    assert_eq!(video_entry.chunk_count, 5);
    assert_eq!(
        video_entry.summary.as_deref(),
        Some("Emma Watson discusses filming.")
    );
    assert_eq!(video_entry.duration_seconds, Some(180));
}

#[tokio::test]
async fn scoped_search_and_graph_introspection_filter_by_node_role_and_relation_kind() {
    let pool = database().await;
    for (id, title) in [("n1", "Alpha"), ("n2", "Beta")] {
        let mut data = json!({"kind":"note","title":title,"plainText":title,"tiptapContent":{"type":"doc","content":[]}});
        crate::spaces::knowledge::model::revision::apply(&mut data).unwrap();
        let revision = data["knowledgeRevision"].as_str().unwrap().to_owned();
        sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) VALUES($1,'space','note',0,0,$2,1,1)")
            .bind(id).bind(data.to_string()).execute(&pool).await.unwrap();
        sqlx::query("DELETE FROM spaces_knowledge_chunks WHERE node_id=$1")
            .bind(id)
            .execute(&pool)
            .await
            .unwrap();
        for (suffix, role, text) in [
            ("sum", "summary", "quantum overview summary"),
            ("seg", "segment", "quantum detailed evidence"),
        ] {
            let locator = crate::spaces::knowledge::indexing::locator(
                &revision,
                json!({"kind":"node"}),
                text,
            );
            sqlx::query("INSERT INTO spaces_knowledge_chunks(id,space_id,node_id,origin,kind,source_revision,text,locator,metadata,created_at,updated_at) VALUES($1,'space',$2,'derived','text',$3,$4,$5,$6,1,1)")
                .bind(format!("{id}-{suffix}"))
                .bind(id)
                .bind(&revision)
                .bind(text)
                .bind(locator.to_string())
                .bind(json!({"role": role}).to_string())
                .execute(&pool)
                .await
                .unwrap();
        }
    }
    sqlx::query("INSERT INTO spaces_relations(id,space_id,source_id,target_id,kind,status,rationale,actor_kind,created_at,updated_at) VALUES('r1','space','n1','n2','supports','active','Confirmed','human',1,1)")
        .execute(&pool)
        .await
        .unwrap();

    let mut tx = pool.begin().await.unwrap();
    let scoped = super::search_knowledge(
        &mut tx,
        "space",
        &super::SearchFilter {
            query: "quantum".into(),
            limit: 10,
            node_ids: Some(vec!["n2".into()]),
            roles: Some(vec!["segment".into()]),
            mode: super::SearchOutputMode::Passages,
        },
    )
    .await
    .unwrap();
    let matches = scoped.as_array().unwrap();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0]["nodeId"], "n2");
    assert_eq!(matches[0]["chunkId"], "n2-seg");
    assert_eq!(matches[0]["role"], "segment");

    let node_survey = super::search_knowledge(
        &mut tx,
        "space",
        &super::SearchFilter {
            query: "quantum | detailed".into(),
            limit: 10,
            node_ids: None,
            roles: None,
            mode: super::SearchOutputMode::Nodes,
        },
    )
    .await
    .unwrap();
    let survey_items = node_survey.as_array().unwrap();
    assert_eq!(survey_items.len(), 2);
    assert!(survey_items
        .iter()
        .all(|entry| entry["matchCount"].as_u64() == Some(2)));

    // Verify long CJK chunks return compact match-centered excerpts instead of full text.
    let long_prefix = "前置背景描述".repeat(45);
    let long_suffix = "后续补充说明".repeat(45);
    let long_text = format!("{long_prefix}关键商业化决策锚点{long_suffix}");
    let n1_rev: String = sqlx::query_scalar(
        "SELECT json_extract(data,'$.knowledgeRevision') FROM spaces_nodes WHERE id='n1'",
    )
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    let long_locator = crate::spaces::knowledge::indexing::locator(
        &n1_rev,
        json!({"kind":"node"}),
        "关键商业化决策锚点",
    );
    sqlx::query("INSERT INTO spaces_knowledge_chunks(id,space_id,node_id,origin,kind,source_revision,text,locator,metadata,created_at,updated_at) VALUES('n1-long','space','n1','derived','text',$1,$2,$3,'{\"role\":\"text\"}',2,2)")
        .bind(&n1_rev)
        .bind(&long_text)
        .bind(long_locator.to_string())
        .execute(&mut *tx)
        .await
        .unwrap();
    let cjk_or = super::search_knowledge(
        &mut tx,
        "space",
        &super::SearchFilter {
            query: "不存在的词 | 锚点".into(),
            limit: 10,
            node_ids: None,
            roles: None,
            mode: super::SearchOutputMode::Passages,
        },
    )
    .await
    .unwrap();
    let cjk_hits = cjk_or.as_array().unwrap();
    assert_eq!(cjk_hits.len(), 1);
    let preview = cjk_hits[0]["preview"].as_str().unwrap();
    assert!(preview.contains("关键商业化决策锚点"));
    assert!(
        preview.chars().count() <= 300,
        "expected compact excerpt, got {} chars",
        preview.chars().count()
    );

    let types = super::list_object_types(&mut tx, "space").await.unwrap();
    let type_ids: Vec<&str> = types["objectTypes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["id"].as_str().unwrap())
        .collect();
    assert!(type_ids.contains(&"system:claim"));
    assert!(type_ids.contains(&"system:decision"));
    assert!(type_ids.contains(&"system:question"));
    assert!(types["validRelationKinds"]
        .as_array()
        .unwrap()
        .iter()
        .any(|value| value == "supports"));

    let relations = super::list_relations(&mut tx, "space", Some("n1"), Some("supports"), 20)
        .await
        .unwrap();
    let relation_items = relations.as_array().unwrap();
    assert_eq!(relation_items.len(), 1);
    assert_eq!(relation_items[0]["sourceTitle"], "Alpha");
    assert_eq!(relation_items[0]["targetTitle"], "Beta");
}
