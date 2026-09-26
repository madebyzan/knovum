use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

use super::super::{commands::execute_in_transaction, edit_contract::ExecuteSpaceEditRequest};
use crate::spaces::db;

async fn database() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("database");
    db::migrate(&pool).await.expect("schema");
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)")
        .execute(&pool).await.expect("space");
    let mut tx = db::begin_write(&pool).await.expect("initialize begin");
    super::super::repository::initialize_empty_in_transaction(&mut tx, "space")
        .await
        .expect("current structure state");
    tx.commit().await.expect("initialize commit");
    pool
}

fn request(title: &str) -> ExecuteSpaceEditRequest {
    serde_json::from_value(serde_json::json!({
        "spaceId": "space", "mutationId": "mutation", "expectedEditRevision": 0,
        "command": {"kind": "section.create-empty", "sectionId": "section", "title": title,
            "frame": {"x": 0, "y": 0, "width": 300, "height": 200}, "rootOrder": 0}
    }))
    .expect("current command")
}

#[tokio::test]
async fn retry_returns_original_receipt_and_changed_request_is_rejected() {
    let pool = database().await;
    let mut tx = db::begin_write(&pool).await.expect("begin");
    let first = execute_in_transaction(&mut tx, request("Section"))
        .await
        .expect("first edit");
    tx.commit().await.expect("commit");
    let mut tx = db::begin_write(&pool).await.expect("retry begin");
    let retry = execute_in_transaction(&mut tx, request("Section"))
        .await
        .expect("retry");
    assert_eq!(
        serde_json::to_value(first).unwrap(),
        serde_json::to_value(retry).unwrap()
    );
    assert!(execute_in_transaction(&mut tx, request("Different"))
        .await
        .err()
        .expect("mutation identity must bind its request")
        .contains("different request"));
    tx.rollback().await.expect("rollback rejected request");
    let counts: (i64, i64, i64) = sqlx::query_as(
        "SELECT revision,(SELECT COUNT(*) FROM spaces_edit_receipts),\
         (SELECT COUNT(*) FROM spaces_structure_nodes WHERE deleted_at IS NULL) FROM spaces_edit_state WHERE space_id='space'"
    ).fetch_one(&pool).await.expect("durable state");
    assert_eq!(counts, (1, 1, 1));
}

#[tokio::test]
async fn rollback_discards_structure_revision_and_receipt_together() {
    let pool = database().await;
    let mut tx = db::begin_write(&pool).await.expect("begin");
    execute_in_transaction(&mut tx, request("Section"))
        .await
        .expect("uncommitted edit");
    tx.rollback().await.expect("simulated caller failure");
    let counts: (i64, i64, i64) = sqlx::query_as(
        "SELECT revision,(SELECT COUNT(*) FROM spaces_edit_receipts),\
         (SELECT COUNT(*) FROM spaces_structure_nodes) FROM spaces_edit_state WHERE space_id='space'",
    )
    .fetch_one(&pool)
    .await
    .expect("durable state");
    assert_eq!(counts, (0, 0, 0));
}

#[tokio::test]
async fn competing_retries_commit_one_edit_and_return_the_same_receipt() {
    let pool = database().await;
    let apply = || async {
        let mut tx = db::begin_write(&pool).await.expect("begin");
        let receipt = execute_in_transaction(&mut tx, request("Section"))
            .await
            .expect("edit");
        tx.commit().await.expect("commit");
        serde_json::to_value(receipt).unwrap()
    };
    let (first, second) = tokio::join!(apply(), apply());
    assert_eq!(first, second);
    let revision: i64 =
        sqlx::query_scalar("SELECT revision FROM spaces_edit_state WHERE space_id='space'")
            .fetch_one(&pool)
            .await
            .expect("revision");
    assert_eq!(revision, 1);
}
