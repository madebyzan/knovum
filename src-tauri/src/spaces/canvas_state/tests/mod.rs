mod persistence;

use super::{contract::CanvasDocumentV1, validate_document};
use sqlx::sqlite::SqlitePoolOptions;

fn document(source: &str) -> CanvasDocumentV1 {
    serde_json::from_str(source).expect("valid test document shape")
}

#[test]
fn validates_current_engine_neutral_document() {
    let value = document(
        r#"{"version":1,"viewport":{"x":0,"y":0,"zoom":1},"nodes":[{"nodeId":"n","frame":{"x":1,"y":2,"width":320,"height":180,"rotation":0},"sizeAuthority":"frame","zIndex":0}]}"#,
    );
    validate_document(&value).expect("current document must validate");
}

#[test]
fn rejects_domain_membership_and_lock_state() {
    let result = serde_json::from_str::<CanvasDocumentV1>(
        r#"{"version":1,"viewport":{"x":0,"y":0,"zoom":1},"nodes":[{"nodeId":"n","frame":{"x":0,"y":0,"width":1,"height":1,"rotation":0},"sizeAuthority":"frame","parentNodeId":"section","zIndex":0,"locked":true}]}"#,
    );
    assert!(result.is_err());
}

#[test]
fn rejects_engine_specific_or_domain_fields() {
    let result = serde_json::from_str::<CanvasDocumentV1>(
        r#"{"version":1,"viewport":{"x":0,"y":0,"zoom":1},"nodes":[],"store":{}}"#,
    );
    assert!(result.is_err());
}

#[test]
fn accepts_only_non_default_relation_presentation_overrides() {
    let value = document(
        r#"{"version":1,"viewport":{"x":0,"y":0,"zoom":1},"nodes":[],"relationStyles":[{"relationId":"relation","color":"blue"}]}"#,
    );
    validate_document(&value).expect("relation presentation override must validate");
    let neutral = serde_json::from_str::<CanvasDocumentV1>(
        r#"{"version":1,"viewport":{"x":0,"y":0,"zoom":1},"nodes":[],"relationStyles":[{"relationId":"relation","color":"neutral"}]}"#,
    );
    assert!(neutral.is_err());
}

#[tokio::test]
async fn gesture_revision_conflict_rolls_back_membership_change() {
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
    sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) VALUES ('section','space','section',0,0,'{\"kind\":\"section\",\"title\":\"Section\"}',1,1)")
        .execute(&pool)
        .await
        .expect("section");
    let initial = document(
        r#"{"version":1,"viewport":{"x":0,"y":0,"zoom":1},"nodes":[{"nodeId":"node","frame":{"x":0,"y":0,"width":100,"height":100,"rotation":0},"sizeAuthority":"frame","zIndex":0}]}"#,
    );
    super::repository::save(&pool, "space", 0, &initial)
        .await
        .expect("initial canvas");
    let request = super::gesture::CommitCanvasGestureRequest {
        space_id: "space".into(),
        changes: vec![crate::spaces::nodes::contracts::NodeDataChange {
            id: "node".into(),
            data: serde_json::from_str(
                r#"{"kind":"note","title":"Node","tiptapContent":{"type":"doc","content":[]},"plainText":"","sectionId":"section"}"#,
            )
            .expect("node data"),
            source_chunks: None,
        }],
        base_revision: 0,
        document: initial,
    };

    super::gesture::commit_in_transaction(&pool, &request)
        .await
        .expect_err("stale revision must reject the whole gesture");

    let data: String = sqlx::query_scalar("SELECT data FROM spaces_nodes WHERE id='node'")
        .fetch_one(&pool)
        .await
        .expect("node data");
    assert!(!data.contains("sectionId"));
}

#[tokio::test]
async fn canvas_revision_conflict_rolls_back_other_application_writes() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("sqlite");
    crate::spaces::db::migrate(&pool)
        .await
        .expect("spaces migrations");
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at,last_opened_at) VALUES ('space','space','Space',1,1,1)")
        .execute(&pool)
        .await
        .expect("space");
    let initial = document(r#"{"version":1,"viewport":{"x":0,"y":0,"zoom":1},"nodes":[]}"#);
    super::repository::save(&pool, "space", 0, &initial)
        .await
        .expect("initial canvas");

    let mut tx = pool.begin().await.expect("transaction");
    sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) VALUES ('node','space','note',0,0,'{\"kind\":\"note\",\"title\":\"Node\",\"tiptapContent\":{\"type\":\"doc\",\"content\":[]},\"plainText\":\"\"}',1,1)")
        .execute(&mut *tx)
        .await
        .expect("node insert");
    let changed = document(
        r#"{"version":1,"viewport":{"x":0,"y":0,"zoom":1},"nodes":[{"nodeId":"node","frame":{"x":0,"y":0,"width":100,"height":100,"rotation":0},"sizeAuthority":"frame","zIndex":0}]}"#,
    );
    let error = super::repository::save_in_transaction(&mut tx, "space", 0, &changed)
        .await
        .expect_err("stale revision must reject the application transaction");
    assert!(error.contains("revision conflict"));
    tx.rollback().await.expect("rollback");

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces_nodes WHERE id='node'")
        .fetch_one(&pool)
        .await
        .expect("node count");
    assert_eq!(count, 0);
}
