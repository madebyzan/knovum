use super::{document, CanvasDocumentV1, SqlitePoolOptions};
use crate::spaces::canvas_state::repository;

async fn database() -> sqlx::SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("sqlite");
    crate::spaces::db::migrate(&pool).await.expect("migrations");
    pool
}

async fn create_space(pool: &sqlx::SqlitePool) {
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at,last_opened_at) VALUES ('space','space','Space',1,1,1)")
        .execute(pool)
        .await
        .expect("space");
}

fn empty_document() -> CanvasDocumentV1 {
    document(r#"{"version":1,"viewport":{"x":0,"y":0,"zoom":1},"nodes":[]}"#)
}

#[tokio::test]
async fn missing_space_is_not_reported_as_a_revision_conflict() {
    let pool = database().await;
    let canvas = empty_document();
    for revision in [0, 1] {
        let error = repository::save(&pool, "space", revision, &canvas)
            .await
            .expect_err("an absent aggregate cannot own a canvas snapshot");
        assert_eq!(error, "Cannot save canvas state for missing Space space");
    }
    assert!(repository::load(&pool, "space")
        .await
        .expect("load")
        .is_none());
}

#[tokio::test]
async fn first_snapshot_is_created_only_after_its_space_exists() {
    let pool = database().await;
    let canvas = empty_document();
    repository::save(&pool, "space", 0, &canvas)
        .await
        .expect_err("draft must not be persisted without its Space");
    create_space(&pool).await;
    assert_eq!(
        repository::save(&pool, "space", 0, &canvas)
            .await
            .expect("first snapshot"),
        1
    );
}

#[tokio::test]
async fn existing_space_still_rejects_stale_initial_and_update_revisions() {
    let pool = database().await;
    create_space(&pool).await;
    let canvas = empty_document();
    repository::save(&pool, "space", 0, &canvas)
        .await
        .expect("initial snapshot");
    repository::save(&pool, "space", 1, &canvas)
        .await
        .expect("second snapshot");
    let changed = document(r#"{"version":1,"viewport":{"x":100,"y":200,"zoom":2},"nodes":[]}"#);
    for revision in [0, 1] {
        let error = repository::save(&pool, "space", revision, &changed)
            .await
            .expect_err("stale writes must not overwrite the authoritative document");
        assert_eq!(error, "Canvas state revision conflict");
    }
    let (stored, revision) = repository::load(&pool, "space")
        .await
        .expect("load")
        .expect("canvas");
    assert_eq!(revision, 2);
    assert_eq!(
        serde_json::to_value(stored).expect("stored"),
        serde_json::to_value(canvas).expect("expected")
    );
}
