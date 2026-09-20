use super::{contract::SpaceEditorDocument, validate_document};
use sqlx::sqlite::SqlitePoolOptions;

fn content_document() -> SpaceEditorDocument {
    serde_json::from_value(serde_json::json!({
        "version": 1, "viewport": {"x":0,"y":0,"zoom":1},
        "structure": {"version":1,"items":[{"id":"node","nodeId":"node","kind":"content","parentId":null,"order":0,"locked":false}]},
        "layout": {"version":1,"items":[{"kind":"content","itemId":"node","transform":{"x":1,"y":2,"rotation":0,"scale":1},"size":{"width":320,"height":180},"sizeAuthority":"frame"}]}
    })).expect("current document")
}
#[test]
fn validates_separate_structure_and_local_layout() { validate_document(&content_document()).expect("valid structure document"); }
#[test]
fn rejects_legacy_flat_documents() {
    assert!(serde_json::from_str::<SpaceEditorDocument>(r#"{"version":1,"viewport":{"x":0,"y":0,"zoom":1},"nodes":[]}"#).is_err());
}
#[test]
fn rejects_engine_fields_and_duplicate_layouts() {
    let mut value = serde_json::to_value(content_document()).expect("serialize");
    value["store"] = serde_json::json!({});
    assert!(serde_json::from_value::<SpaceEditorDocument>(value).is_err());
    let mut document = content_document(); document.layout.items.push(document.layout.items[0].clone());
    assert!(validate_document(&document).is_err());
}
#[test]
fn relation_colors_are_non_default_and_have_unique_identities() {
    let mut value = serde_json::to_value(SpaceEditorDocument::empty()).expect("serialize");
    value["relationStyles"] = serde_json::json!([{"relationId":"r","color":"blue"}]);
    validate_document(&serde_json::from_value(value.clone()).expect("style")).expect("valid color");
    value["relationStyles"][0]["color"] = serde_json::json!("neutral");
    assert!(serde_json::from_value::<SpaceEditorDocument>(value).is_err());
}
async fn database() -> sqlx::SqlitePool {
    let pool = SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.expect("sqlite");
    crate::spaces::db::migrate(&pool).await.expect("migrations");
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at,last_opened_at) VALUES('space','space','Space',1,1,1)").execute(&pool).await.expect("space");
    pool
}
#[tokio::test]
async fn stale_revision_rolls_back_content_and_structure_together() {
    let pool = database().await;
    super::repository::save(&pool,"space",0,&SpaceEditorDocument::empty()).await.expect("initial state");
    let mut tx = pool.begin().await.expect("transaction");
    sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) VALUES('node','space','note',0,0,'{\"kind\":\"note\",\"title\":\"Node\",\"tiptapContent\":{\"type\":\"doc\",\"content\":[]},\"plainText\":\"\"}',1,1)")
        .execute(&mut *tx).await.expect("staged content");
    let error = super::repository::save_in_transaction(&mut tx,"space",0,&content_document()).await.expect_err("stale revision");
    assert!(error.contains("revision conflict"));
    tx.rollback().await.expect("rollback");
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces_nodes").fetch_one(&pool).await.expect("count");
    assert_eq!(count,0);
    let (document,revision) = super::repository::load(&pool,"space").await.expect("load").expect("state");
    assert_eq!(revision,1); assert!(document.structure.items.is_empty());
}
#[tokio::test]
async fn camera_persistence_does_not_advance_the_edit_revision() {
    let pool = database().await;
    super::repository::save(&pool,"space",0,&SpaceEditorDocument::empty()).await.expect("initial state");
    let mut tx = pool.begin().await.expect("transaction");
    super::viewport::save_in_transaction(&mut tx,"space",&super::contract::CanvasViewport { x:120.0,y:-40.0,zoom:0.8 }).await.expect("camera");
    tx.commit().await.expect("commit");
    let (document,revision) = super::repository::load(&pool,"space").await.expect("load").expect("state");
    assert_eq!(revision,1); assert_eq!(document.viewport.x,120.0); assert_eq!(document.viewport.zoom,0.8);
}
