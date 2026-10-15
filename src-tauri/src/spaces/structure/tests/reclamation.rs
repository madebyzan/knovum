use std::collections::BTreeSet;

use serde_json::json;
use sqlx::SqlitePool;

use super::super::{commands::execute_in_transaction, reclamation::reclaim_in_transaction};
use super::transactions::database;
use crate::spaces::db;

/// Creates an image card and deletes it, leaving the soft-deleted rows undo restores from.
async fn deleted_photo() -> SqlitePool {
    let pool = database().await;
    let mut tx = db::begin_write(&pool).await.unwrap();
    crate::spaces::nodes::tests::insert_many_in_transaction(
        &mut tx,
        serde_json::from_value(json!({
            "spaceId":"space","mutationId":"create","targetParentId":null,
            "nodes":[{"id":"photo","kind":"image","initialX":0,"initialY":0,"width":300,"height":200,
                "sizingMode":"frame","sourceChunks":[],
                "data":{"kind":"image","title":"Photo","assetPath":"/managed/photo.png","fileName":"photo.png"}}]
        }))
        .unwrap(),
        None,
    )
    .await
    .unwrap();
    let structure_node_id: String =
        sqlx::query_scalar("SELECT id FROM spaces_structure_nodes WHERE node_id='photo'")
            .fetch_one(&mut *tx)
            .await
            .unwrap();
    execute_in_transaction(
        &mut tx,
        serde_json::from_value(json!({
            "spaceId":"space","mutationId":"delete","expectedEditRevision":1,
            "command":{"kind":"structure.delete-subtrees","rootIds":[structure_node_id]}
        }))
        .unwrap(),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    pool
}

async fn reclaim(pool: &SqlitePool) -> BTreeSet<String> {
    let mut tx = db::begin_write(pool).await.unwrap();
    let asset_candidates = reclaim_in_transaction(&mut tx).await.unwrap();
    tx.commit().await.unwrap();
    asset_candidates
}

/// What pruning past the history limit, or a new edit discarding the redo branch, does.
async fn lose_undo(pool: &SqlitePool) {
    sqlx::query("DELETE FROM spaces_edit_history")
        .execute(pool)
        .await
        .unwrap();
}

async fn retained_rows(pool: &SqlitePool) -> (i64, i64, i64) {
    sqlx::query_as(
        "SELECT (SELECT COUNT(*) FROM spaces_nodes WHERE id='photo'), \
         (SELECT COUNT(*) FROM spaces_structure_nodes WHERE node_id='photo'), \
         (SELECT COUNT(*) FROM spaces_structure_layouts WHERE structure_node_id IN \
           (SELECT id FROM spaces_structure_nodes WHERE node_id='photo'))",
    )
    .fetch_one(pool)
    .await
    .unwrap()
}

#[tokio::test]
async fn deleted_content_lives_exactly_as_long_as_undo_can_restore_it() {
    let pool = deleted_photo().await;

    assert!(reclaim(&pool).await.is_empty());
    assert_eq!(retained_rows(&pool).await, (1, 1, 1));

    lose_undo(&pool).await;

    assert_eq!(
        reclaim(&pool).await,
        BTreeSet::from(["/managed/photo.png".to_owned()])
    );
    assert_eq!(retained_rows(&pool).await, (0, 0, 0));
}

#[tokio::test]
async fn a_live_portal_keeps_the_content_it_targets() {
    let pool = deleted_photo().await;
    sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) VALUES('portal','space','portal',0,0,'{\"id\":\"portal\",\"kind\":\"portal\",\"targetSpaceId\":\"space\",\"targetNodeId\":\"photo\"}',1,1)")
        .execute(&pool).await.unwrap();
    lose_undo(&pool).await;

    assert!(reclaim(&pool).await.is_empty());
    let photo: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces_nodes WHERE id='photo'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(photo, 1);
}
