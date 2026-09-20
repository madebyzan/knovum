use sqlx::sqlite::SqlitePoolOptions;

mod invalidation;

use super::{
    contracts::{
        DeleteCanvasSelectionRequest, RestoreCanvasSelectionRequest,
        SetImportedCanvasNodesDeletedRequest,
    },
    deletion::delete_in_transaction,
    lifecycle::{restore_in_transaction, set_imported_nodes_deleted_in_transaction},
};

#[tokio::test]
async fn relation_failure_rolls_back_node_and_canvas_changes() {
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
    let initial = serde_json::from_str(r#"{"version":1,"viewport":{"x":0,"y":0,"zoom":1},"nodes":[{"nodeId":"node","frame":{"x":0,"y":0,"width":100,"height":100,"rotation":0},"sizeAuthority":"frame","zIndex":0}]}"#)
        .expect("canvas");
    super::super::repository::save(&pool, "space", 0, &initial)
        .await
        .expect("initial state");
    let request = DeleteCanvasSelectionRequest {
        space_id: "space".into(),
        node_ids: vec!["node".into()],
        relation_ids: vec!["missing".into()],
        survivor_changes: vec![],
        base_revision: 1,
        document: serde_json::from_str(
            r#"{"version":1,"viewport":{"x":0,"y":0,"zoom":1},"nodes":[]}"#,
        )
        .expect("canvas"),
    };

    delete_in_transaction(&pool, &request)
        .await
        .expect_err("relation must fail");

    let deleted_at: Option<i64> =
        sqlx::query_scalar("SELECT deleted_at FROM spaces_nodes WHERE id='node'")
            .fetch_one(&pool)
            .await
            .expect("deleted state");
    let revision: i64 =
        sqlx::query_scalar("SELECT revision FROM spaces_canvas_state WHERE space_id='space'")
            .fetch_one(&pool)
            .await
            .expect("revision");
    assert_eq!(deleted_at, None);
    assert_eq!(revision, 1);
}

#[tokio::test]
async fn restore_rolls_back_when_canvas_revision_conflicts() {
    let pool = seeded_pool(true).await;
    let request = RestoreCanvasSelectionRequest {
        space_id: "space".into(),
        node_ids: vec!["node".into()],
        relation_ids: vec![],
        survivor_changes: vec![],
        base_revision: 0,
        document: document_with_node(),
    };

    restore_in_transaction(&pool, &request)
        .await
        .expect_err("stale canvas revision must reject restore");

    assert_eq!(node_deleted_at(&pool).await, Some(2));
    assert_eq!(canvas_revision(&pool).await, 1);
}

#[tokio::test]
async fn imported_node_deletion_rolls_back_when_canvas_revision_conflicts() {
    let pool = seeded_pool(false).await;
    let request = SetImportedCanvasNodesDeletedRequest {
        space_id: "space".into(),
        node_ids: vec!["node".into()],
        relation_ids: Vec::new(),
        deleted: true,
        base_revision: 0,
        document: empty_document(),
    };

    set_imported_nodes_deleted_in_transaction(&pool, &request)
        .await
        .expect_err("stale canvas revision must reject imported node deletion");

    assert_eq!(node_deleted_at(&pool).await, None);
    assert_eq!(canvas_revision(&pool).await, 1);
}

async fn seeded_pool(deleted: bool) -> sqlx::SqlitePool {
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
    sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at,deleted_at) VALUES ('node','space','note',0,0,'{\"kind\":\"note\",\"title\":\"Node\",\"tiptapContent\":{\"type\":\"doc\",\"content\":[]},\"plainText\":\"\"}',1,1,$1)")
        .bind(deleted.then_some(2_i64))
        .execute(&pool)
        .await
        .expect("node");
    super::super::repository::save(&pool, "space", 0, &document_with_node())
        .await
        .expect("initial canvas");
    pool
}

fn document_with_node() -> super::super::SpaceEditorDocument {
    serde_json::from_str(r#"{"version":1,"viewport":{"x":0,"y":0,"zoom":1},"nodes":[{"nodeId":"node","frame":{"x":0,"y":0,"width":100,"height":100,"rotation":0},"sizeAuthority":"frame","zIndex":0}]}"#)
        .expect("canvas")
}

fn empty_document() -> super::super::SpaceEditorDocument {
    serde_json::from_str(r#"{"version":1,"viewport":{"x":0,"y":0,"zoom":1},"nodes":[]}"#)
        .expect("canvas")
}

async fn node_deleted_at(pool: &sqlx::SqlitePool) -> Option<i64> {
    sqlx::query_scalar("SELECT deleted_at FROM spaces_nodes WHERE id='node'")
        .fetch_one(pool)
        .await
        .expect("deleted state")
}

async fn canvas_revision(pool: &sqlx::SqlitePool) -> i64 {
    sqlx::query_scalar("SELECT revision FROM spaces_canvas_state WHERE space_id='space'")
        .fetch_one(pool)
        .await
        .expect("canvas revision")
}
