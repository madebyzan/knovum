use sqlx::sqlite::SqlitePoolOptions;

use super::{delete_in_transaction, DeleteCanvasSelectionRequest};

#[tokio::test]
async fn selection_deletion_commits_retrieval_invalidation_with_canvas_state() {
    let pool = seeded_pool().await;

    delete_in_transaction(&pool, &deletion_request(1))
        .await
        .expect("delete selection");

    assert_eq!(pending_invalidations(&pool).await, 1);
    let revision: i64 =
        sqlx::query_scalar("SELECT revision FROM spaces_canvas_state WHERE space_id='space'")
            .fetch_one(&pool)
            .await
            .expect("canvas revision");
    assert_eq!(revision, 2);
}

#[tokio::test]
async fn canvas_conflict_rolls_back_retrieval_invalidation() {
    let pool = seeded_pool().await;

    delete_in_transaction(&pool, &deletion_request(0))
        .await
        .expect_err("stale canvas revision");

    assert_eq!(pending_invalidations(&pool).await, 0);
    let deleted_at: Option<i64> =
        sqlx::query_scalar("SELECT deleted_at FROM spaces_nodes WHERE id='node'")
            .fetch_one(&pool)
            .await
            .expect("node lifecycle");
    assert_eq!(deleted_at, None);
}

async fn seeded_pool() -> sqlx::SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("sqlite");
    crate::spaces::db::migrate(&pool).await.expect("migrations");
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at,last_opened_at) VALUES ('space','space','Space',1,1,1)")
        .execute(&pool)
        .await
        .expect("space");
    sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) VALUES ('node','space','note',0,0,'{\"kind\":\"note\",\"title\":\"Node\",\"tiptapContent\":{\"type\":\"doc\",\"content\":[]},\"plainText\":\"\"}',1,1)")
        .execute(&pool)
        .await
        .expect("node");
    super::super::super::repository::save(&pool, "space", 0, &document_with_node())
        .await
        .expect("initial canvas");
    pool
}

fn deletion_request(base_revision: i64) -> DeleteCanvasSelectionRequest {
    DeleteCanvasSelectionRequest {
        space_id: "space".into(),
        node_ids: vec!["node".into()],
        relation_ids: vec![],
        survivor_changes: vec![],
        base_revision,
        document: empty_document(),
    }
}

fn document_with_node() -> super::super::super::CanvasDocumentV1 {
    serde_json::from_str(r#"{"version":1,"viewport":{"x":0,"y":0,"zoom":1},"nodes":[{"nodeId":"node","frame":{"x":0,"y":0,"width":100,"height":100,"rotation":0},"sizeAuthority":"frame","zIndex":0}]}"#)
        .expect("canvas")
}

fn empty_document() -> super::super::super::CanvasDocumentV1 {
    serde_json::from_str(r#"{"version":1,"viewport":{"x":0,"y":0,"zoom":1},"nodes":[]}"#)
        .expect("canvas")
}

async fn pending_invalidations(pool: &sqlx::SqlitePool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM spaces_retrieval_invalidations")
        .fetch_one(pool)
        .await
        .expect("pending invalidations")
}
