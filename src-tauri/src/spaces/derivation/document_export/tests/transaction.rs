use super::super::{lifecycle, ExportDocumentRequest};
use crate::spaces::{db, nodes::contracts::InsertNodesRequest, structure::repository};
use serde_json::json;

async fn database() -> (sqlx::SqlitePool, String) {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    db::migrate(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)").execute(&pool).await.unwrap();
    let mut tx = db::begin_write(&pool).await.unwrap();
    repository::initialize_empty_in_transaction(&mut tx, "space")
        .await
        .unwrap();
    let mut data = json!({"kind":"markdown","title":"Source","text":"Research content"});
    crate::spaces::knowledge::indexing::revision::apply(&mut data).unwrap();
    let revision = data["knowledgeRevision"].as_str().unwrap().to_owned();
    let request: InsertNodesRequest = serde_json::from_value(json!({
        "spaceId":"space","mutationId":"source-create","expectedEditRevision":0,"targetParentId":null,
        "nodes":[{"id":"source","kind":"markdown","initialX":0,"initialY":0,"width":300,"height":200,"sizeAuthority":"frame","data":data,"sourceChunks":[]}]
    })).unwrap();
    crate::spaces::nodes::insert_many_in_transaction(&mut tx, request)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    (pool, revision)
}

fn request(revision: &str) -> ExportDocumentRequest {
    serde_json::from_value(json!({"id":"export","spaceId":"space","sourceNodeId":"source","sourceRevision":revision,"outputNodeId":"output","expectedEditRevision":1,"targetParentId":null,"position":{"x":30,"y":40},"size":{"width":300,"height":200}})).unwrap()
}

#[tokio::test]
async fn export_output_structure_and_derivation_commit_together() {
    let (pool, revision) = database().await;
    let mut tx = db::begin_write(&pool).await.unwrap();
    let prepared = lifecycle::prepare_in_transaction(
        &mut tx,
        &request(&revision),
        "export-document-as-txt",
        "text",
        "txt",
        "/owned/document.txt",
    )
    .await
    .unwrap();
    assert_eq!(prepared.source.title, "Source");
    tx.commit().await.unwrap();
    let counts: (i64,i64,i64,i64) = sqlx::query_as("SELECT (SELECT COUNT(*) FROM spaces_nodes WHERE id='output'),(SELECT COUNT(*) FROM spaces_structure_nodes WHERE node_id='output'),(SELECT COUNT(*) FROM spaces_knowledge_derivations WHERE id='export'),(SELECT revision FROM spaces_edit_state WHERE space_id='space')").fetch_one(&pool).await.unwrap();
    assert_eq!(counts, (1, 1, 1, 2));
}

#[tokio::test]
async fn derivation_insert_failure_rolls_back_output_and_edit_revision() {
    let (pool, revision) = database().await;
    let mut tx = db::begin_write(&pool).await.unwrap();
    assert!(lifecycle::prepare_in_transaction(
        &mut tx,
        &request(&revision),
        "invalid-action",
        "text",
        "txt",
        "/owned/document.txt"
    )
    .await
    .is_err());
    tx.rollback().await.unwrap();
    let counts: (i64,i64,i64) = sqlx::query_as("SELECT (SELECT COUNT(*) FROM spaces_nodes WHERE id='output'),(SELECT COUNT(*) FROM spaces_edit_receipts WHERE mutation_id='export:output'),(SELECT revision FROM spaces_edit_state WHERE space_id='space')").fetch_one(&pool).await.unwrap();
    assert_eq!(counts, (0, 0, 1));
}

#[tokio::test]
async fn stale_source_cannot_create_an_output() {
    let (pool, _) = database().await;
    let mut tx = db::begin_write(&pool).await.unwrap();
    assert!(lifecycle::prepare_in_transaction(
        &mut tx,
        &request("stale"),
        "export-document-as-txt",
        "text",
        "txt",
        "/owned/document.txt"
    )
    .await
    .err()
    .unwrap()
    .contains("source changed"));
    tx.rollback().await.unwrap();
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces_nodes WHERE id='output'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
}
