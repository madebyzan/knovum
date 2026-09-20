use std::collections::{BTreeSet, HashSet};

use sqlx::sqlite::SqlitePoolOptions;

use super::super::{referenced_paths_for_nodes, unreferenced_paths};

const BACKGROUND_PATH: &str = "/managed/section-background.png";

#[tokio::test]
async fn canvas_background_remains_referenced_through_soft_deletion() {
    let pool = seeded_pool().await;
    save_document(&pool, 0, document_with_background()).await;
    assert_eq!(unreferenced(&pool).await, Vec::<String>::new());

    sqlx::query("UPDATE spaces_nodes SET deleted_at=2 WHERE id='section'")
        .execute(&pool)
        .await
        .expect("soft delete section");
    save_document(&pool, 1, empty_document()).await;

    let mut tx = pool.begin().await.expect("transaction");
    let referenced =
        referenced_paths_for_nodes(&mut tx, "space", &HashSet::from(["section".to_owned()]))
            .await
            .expect("deleted node references");
    tx.rollback().await.expect("rollback read transaction");
    assert_eq!(referenced, BTreeSet::from([BACKGROUND_PATH.to_owned()]));
    assert_eq!(unreferenced(&pool).await, Vec::<String>::new());

    sqlx::query("DELETE FROM spaces_nodes WHERE id='section'")
        .execute(&pool)
        .await
        .expect("permanent delete section");
    assert_eq!(unreferenced(&pool).await, vec![BACKGROUND_PATH.to_owned()]);
}

#[tokio::test]
async fn deleting_space_releases_canvas_background() {
    let pool = seeded_pool().await;
    save_document(&pool, 0, document_with_background()).await;

    sqlx::query("DELETE FROM spaces WHERE id='space'")
        .execute(&pool)
        .await
        .expect("delete space");

    assert_eq!(unreferenced(&pool).await, vec![BACKGROUND_PATH.to_owned()]);
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
    sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) VALUES ('section','space','section',0,0,'{\"kind\":\"section\",\"title\":\"Section\"}',1,1)")
        .execute(&pool)
        .await
        .expect("section");
    pool
}

async fn save_document(
    pool: &sqlx::SqlitePool,
    base_revision: i64,
    document: crate::spaces::canvas_state::SpaceEditorDocument,
) {
    let mut tx = pool.begin().await.expect("transaction");
    crate::spaces::canvas_state::save_in_transaction(&mut tx, "space", base_revision, &document)
        .await
        .expect("save canvas");
    tx.commit().await.expect("commit canvas");
}

async fn unreferenced(pool: &sqlx::SqlitePool) -> Vec<String> {
    unreferenced_paths(pool, BTreeSet::from([BACKGROUND_PATH.to_owned()]))
        .await
        .expect("asset references")
}

fn document_with_background() -> crate::spaces::canvas_state::SpaceEditorDocument {
    serde_json::from_str(&format!(
        r#"{{"version":1,"viewport":{{"x":0,"y":0,"zoom":1}},"nodes":[{{"nodeId":"section","frame":{{"x":0,"y":0,"width":100,"height":100,"rotation":0}},"sizeAuthority":"frame","appearance":{{"background":"{BACKGROUND_PATH}","backgroundStyle":"cover"}},"zIndex":0}}]}}"#
    ))
    .expect("canvas")
}

fn empty_document() -> crate::spaces::canvas_state::SpaceEditorDocument {
    serde_json::from_str(r#"{"version":1,"viewport":{"x":0,"y":0,"zoom":1},"nodes":[]}"#)
        .expect("canvas")
}
