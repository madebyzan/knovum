//! The contract a record of an edit has to keep: it names the rows the edit touched, a replay puts
//! exactly those back, and everything else in the Space stays as it is.

use serde_json::json;
use sqlx::SqlitePool;

use super::super::{commands::execute_in_transaction, history, repository};
use super::transactions::database;
use crate::spaces::db;

async fn section(pool: &SqlitePool, index: u32, revision: i64) {
    let mut tx = db::begin_write(pool).await.unwrap();
    execute_in_transaction(
        &mut tx,
        serde_json::from_value(json!({
            "spaceId":"space","mutationId":format!("create-{index}"),"expectedEditRevision":revision,
            "command":{"kind":"section.create-empty","sectionId":format!("section-{index}"),
                "title":format!("Area {index}"),
                "frame":{"x":f64::from(index) * 400.0,"y":0,"width":300,"height":200},
                "rootOrder":index}
        }))
        .unwrap(),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
}

async fn rename(pool: &SqlitePool, revision: i64) {
    let mut tx = db::begin_write(pool).await.unwrap();
    execute_in_transaction(
        &mut tx,
        serde_json::from_value(json!({
            "spaceId":"space","mutationId":"rename","expectedEditRevision":revision,
            "command":{"kind":"container.rename","structureNodeId":"section-0","title":"Renamed"}
        }))
        .unwrap(),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
}

async fn renamed_record_bytes(sections: u32) -> i64 {
    let pool = database().await;
    for index in 0..sections {
        section(&pool, index, i64::from(index)).await;
    }
    rename(&pool, i64::from(sections)).await;
    sqlx::query_scalar("SELECT length(patch) FROM spaces_edit_history ORDER BY id DESC LIMIT 1")
        .fetch_one(&pool)
        .await
        .unwrap()
}

/// The property that makes a hundred remembered edits affordable, stated as a measurement: the same
/// rename costs the same bytes in a Space thirty times larger.
#[tokio::test]
async fn a_record_measures_the_edit_and_not_the_space() {
    let small = renamed_record_bytes(2).await;
    let large = renamed_record_bytes(60).await;
    assert_eq!(small, large);
    assert!(small < 1024, "one rename took {small} bytes");
}

async fn undo(pool: &SqlitePool, revision: i64, redo: bool) -> i64 {
    let mut tx = db::begin_write(pool).await.unwrap();
    let (snapshot, current) = repository::load_in_transaction(&mut tx, "space")
        .await
        .unwrap();
    assert_eq!(current, revision);
    let next = history::apply(&mut tx, "space", current, &snapshot, redo)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    next
}

async fn title(pool: &SqlitePool, structure_node_id: &str) -> String {
    sqlx::query_scalar("SELECT title FROM spaces_structure_nodes WHERE id=$1")
        .bind(structure_node_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

#[tokio::test]
async fn a_replay_writes_the_named_rows_and_leaves_the_rest_untouched() {
    let pool = database().await;
    for index in 0..3 {
        section(&pool, index, i64::from(index)).await;
    }
    rename(&pool, 3).await;
    sqlx::query(
        "CREATE TABLE rewritten(id TEXT); \
         CREATE TRIGGER structure_rewritten AFTER UPDATE ON spaces_structure_nodes \
         BEGIN INSERT INTO rewritten VALUES(NEW.id); END;",
    )
    .execute(&pool)
    .await
    .unwrap();

    assert_eq!(undo(&pool, 4, false).await, 5);
    assert_eq!(title(&pool, "section-0").await, "Area 0");
    assert_eq!(undo(&pool, 5, true).await, 6);
    assert_eq!(title(&pool, "section-0").await, "Renamed");

    let rewritten: Vec<String> = sqlx::query_scalar("SELECT DISTINCT id FROM rewritten")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(rewritten, ["section-0"]);
}

/// A card resized to fit its media after the edit must keep that size when an unrelated move is
/// undone: the record says the edit did not touch the size, so the replay does not either.
#[tokio::test]
async fn undo_keeps_a_size_the_edit_never_changed() {
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
    tx.commit().await.unwrap();
    let placed = layout(&pool, &structure_node_id).await;

    let mut tx = db::begin_write(&pool).await.unwrap();
    execute_in_transaction(
        &mut tx,
        serde_json::from_value(json!({
            "spaceId":"space","mutationId":"move","expectedEditRevision":1,
            "command":{"kind":"content.update-layout","structureNodeId":structure_node_id,
                "transform":{"x":1200,"y":800,"rotation":0,"scale":1},"size":null,"sizingMode":null}
        }))
        .unwrap(),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    // What fitting a card to its media does between edits: it rewrites the size and nothing else.
    sqlx::query(
        "UPDATE spaces_structure_layouts SET data=json_set(data,'$.size.width',640,'$.size.height',480) \
         WHERE space_id='space' AND structure_node_id=$1",
    )
    .bind(&structure_node_id)
    .execute(&pool)
    .await
    .unwrap();

    undo(&pool, 2, false).await;

    let restored = layout(&pool, &structure_node_id).await;
    assert_eq!(restored["size"], json!({"width":640.0,"height":480.0}));
    assert_eq!(restored["transform"], placed["transform"]);
}

async fn layout(pool: &SqlitePool, structure_node_id: &str) -> serde_json::Value {
    let encoded: String = sqlx::query_scalar(
        "SELECT data FROM spaces_structure_layouts WHERE space_id='space' AND structure_node_id=$1",
    )
    .bind(structure_node_id)
    .fetch_one(pool)
    .await
    .unwrap();
    serde_json::from_str(&encoded).unwrap()
}
