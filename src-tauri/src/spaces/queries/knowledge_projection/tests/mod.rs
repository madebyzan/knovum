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
    crate::spaces::knowledge::indexing::revision::apply(&mut data).unwrap();
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
        assert!(super::get_node(&mut tx, "space", id).await.is_err(), "{id}");
    }
    let node = super::get_node(&mut tx, "space", "a").await.unwrap();
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
async fn search_actions_and_mcp_share_chinese_search_and_revision_aware_results() {
    let pool = database().await;
    let data = json!({"kind":"note","title":"Space evidence","plainText":"画布有图片资料","tiptapContent":{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"画布有图片资料"}]}]}});
    let mut normalized = data;
    normalized["knowledgeRevision"] = Value::String(
        crate::spaces::knowledge::indexing::revision::authoritative(&normalized)
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
    let expected = super::search_knowledge(&mut tx, "space", "图片", 20)
        .await
        .unwrap();
    assert_eq!(expected.as_array().unwrap().len(), 1);
    let action = crate::spaces::knowledge::action::search(
        &mut tx,
        "space",
        &json!({"query":"图片","limit":20}),
    )
    .await
    .unwrap();
    assert_eq!(action, expected);
    assert_eq!(action[0]["nodeId"], "note");
    assert_eq!(action[0]["sourceRevision"], revision);
    assert_eq!(action[0]["locator"], locator);
    assert!(super::search_knowledge(&mut tx, "other", "图片", 20)
        .await
        .unwrap()
        .as_array()
        .unwrap()
        .is_empty());
    assert!(super::search_knowledge(&mut tx, "space", " ", 20)
        .await
        .is_err());
    let direct = super::get_node(&mut tx, "space", "note").await.unwrap();
    let action =
        crate::spaces::knowledge::action::read_node(&mut tx, "space", &json!({"nodeId":"note"}))
            .await
            .unwrap();
    assert_eq!(direct, action);
    assert_eq!(direct["data"]["plainText"], "画布有图片资料");
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
        super::search_knowledge(&mut tx, "space", "图片", 20)
            .await
            .unwrap(),
        Value::Array(vec![])
    );
}
