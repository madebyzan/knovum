pub(in crate::spaces) use super::insert::insert_many_in_transaction;

use serde_json::json;

use crate::spaces::{db, nodes::contracts::InsertNodesRequest, structure};

/// One node as the canvas asks for it, which is what every fixture that needs something on the
/// canvas is really building.
pub(in crate::spaces) fn insert_request(
    space_id: &str,
    mutation_id: &str,
    node_id: &str,
    kind: &str,
    data: serde_json::Value,
) -> InsertNodesRequest {
    serde_json::from_value(json!({
        "spaceId":space_id,"mutationId":mutation_id,"targetParentId":null,
        "nodes":[{"id":node_id,"kind":kind,"initialX":0,"initialY":0,"width":320,"height":180,
            "sizingMode":"content","sourceChunks":[],"data":data}]
    }))
    .expect("a node insertion request")
}

/// Inserts the node a fixture will derive from.
pub(in crate::spaces) async fn insert_source_in_transaction(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    node_id: &str,
    kind: &str,
    data: serde_json::Value,
) -> Result<(), String> {
    let request = insert_request(space_id, &format!("{node_id}-create"), node_id, kind, data);
    insert_many_in_transaction(tx, request, None)
        .await
        .map(|_| ())
}

/// The plain text source a narration or a document is made from.
pub(in crate::spaces) fn source_note(title: &str) -> serde_json::Value {
    json!({"kind":"note","title":title,"plainText":"Hello",
        "tiptapContent":{"type":"doc","content":[{"type":"paragraph"}]}})
}

async fn initialized_space() -> sqlx::SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    db::migrate(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)")
        .execute(&pool).await.unwrap();
    let mut tx = db::begin_write(&pool).await.unwrap();
    structure::repository::initialize_empty_in_transaction(&mut tx, "space")
        .await
        .unwrap();
    tx.commit().await.unwrap();
    pool
}

fn import(name: &str) -> InsertNodesRequest {
    insert_request(
        "space",
        name,
        name,
        "image",
        json!({"kind":"image","title":name,"assetPath":format!("/managed/{name}.png"),
            "fileName":format!("{name}.png")}),
    )
}

/// Dropping several files runs their imports in parallel. Each request is built from the Space the
/// user dropped onto, so none of them can know the revision its siblings commit.
#[tokio::test]
async fn parallel_imports_append_without_a_caller_revision() {
    let pool = initialized_space().await;
    let append = |request| {
        let pool = pool.clone();
        async move {
            let mut tx = db::begin_write(&pool).await.unwrap();
            let revision = insert_many_in_transaction(&mut tx, request, None)
                .await
                .expect("import appends to the Space it is committed against");
            tx.commit().await.unwrap();
            revision
        }
    };
    let (first, second, third) = tokio::join!(
        append(import("first")),
        append(import("second")),
        append(import("third"))
    );
    let mut applied = [first, second, third];
    applied.sort_unstable();
    assert_eq!(applied, [1, 2, 3]);
    let committed: Vec<String> =
        sqlx::query_scalar("SELECT id FROM spaces_nodes WHERE space_id='space' ORDER BY id")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(committed, ["first", "second", "third"]);
    let (revision, structure_nodes): (i64, i64) = sqlx::query_as(
        "SELECT revision,(SELECT COUNT(*) FROM spaces_structure_nodes WHERE space_id='space' \
         AND deleted_at IS NULL) FROM spaces_edit_state WHERE space_id='space'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!((revision, structure_nodes), (3, 3));
}

#[tokio::test]
async fn user_edits_receive_a_rust_owned_revision() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
    crate::spaces::db::migrate(&pool).await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    let mut data = json!({"kind":"markdown","title":"Note","text":"Updated text","knowledgeRevision":"obsolete-ui-hash"});
    crate::spaces::knowledge::validation::normalize_node(&mut tx, "space", "markdown", &mut data)
        .await
        .unwrap();
    assert_eq!(
        data["knowledgeRevision"].as_str(),
        crate::spaces::knowledge::model::revision::authoritative(&data)
            .unwrap()
            .as_deref()
    );
    assert_ne!(data["knowledgeRevision"], "obsolete-ui-hash");
}

#[tokio::test]
async fn resetting_image_edits_replaces_the_visual_job_through_the_node_update_pipeline() {
    use crate::spaces::{assets::enrichment::jobs, nodes::contracts::NodeContentChange};
    let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
    crate::spaces::db::migrate(&pool).await.unwrap();
    let adjustments: serde_json::Map<String, serde_json::Value> = [
        "exposure",
        "contrast",
        "highlights",
        "shadows",
        "whites",
        "blacks",
        "saturation",
        "warmth",
        "sharpness",
        "vignette",
        "vibrance",
        "tint",
        "clarity",
        "grain",
        "softLight",
        "glow",
    ]
    .into_iter()
    .map(|key| (key.to_owned(), json!(0.0)))
    .collect();
    let mut before = json!({
        "kind":"image", "title":"Picture", "sourceRevision":"asset-1",
        "assetPath":"/assets/picture.png", "fileName":"picture.png",
        "imageEdit":{
            "version":1, "crop":{"x":0.0,"y":0.0,"width":1.0,"height":1.0},
            "rotation":90.0,"flipHorizontal":false,"flipVertical":false,
            "adjustments":adjustments
        }
    });
    crate::spaces::knowledge::model::revision::apply(&mut before).unwrap();
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)")
        .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) VALUES('node','space','image',0,0,$1,1,1)")
        .bind(before.to_string()).execute(&pool).await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    jobs::intent::synchronize_for_node(&mut tx, "space", "node", &before, 1, None)
        .await
        .unwrap();
    let mut after = before.clone();
    after.as_object_mut().unwrap().remove("imageEdit");
    crate::spaces::knowledge::validation::normalize_node(&mut tx, "space", "image", &mut after)
        .await
        .unwrap();
    super::update_many_in_transaction(
        &mut tx,
        "space",
        &[NodeContentChange {
            id: "node".into(),
            expected_content: before,
            data: after.clone(),
            source_chunks: None,
        }],
        "human",
        2,
        None,
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    let cancelled: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM spaces_enrichment_jobs WHERE status='cancelled'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(cancelled, 1);
    let pending: Vec<String> = sqlx::query_scalar("SELECT source_revision FROM spaces_enrichment_jobs WHERE job_type='analyze_visual' AND status='pending'")
        .fetch_all(&pool).await.unwrap();
    assert_eq!(
        pending,
        vec![jobs::revision::visual_revision(&after).unwrap().unwrap()]
    );
}
