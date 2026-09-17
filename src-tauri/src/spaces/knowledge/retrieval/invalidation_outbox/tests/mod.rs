use sqlx::sqlite::SqlitePoolOptions;

use super::{enqueue_nodes, enqueue_space};

#[tokio::test]
async fn space_invalidation_supersedes_node_invalidations() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("sqlite");
    crate::spaces::db::migrate(&pool).await.expect("migrations");
    let mut tx = pool.begin().await.expect("transaction");
    enqueue_nodes(&mut tx, "space", &["a".into(), "b".into()], 1)
        .await
        .expect("node invalidations");
    enqueue_space(&mut tx, "space", 2)
        .await
        .expect("space invalidation");
    enqueue_nodes(&mut tx, "space", &["c".into()], 3)
        .await
        .expect("redundant node invalidation");
    tx.commit().await.expect("commit");

    let rows: Vec<(String, Option<String>)> =
        sqlx::query_as("SELECT space_id,node_id FROM spaces_retrieval_invalidations")
            .fetch_all(&pool)
            .await
            .expect("pending invalidations");
    assert_eq!(rows, vec![("space".into(), None)]);
}

#[tokio::test]
async fn space_invalidation_survives_space_deletion() {
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
    let mut tx = pool.begin().await.expect("transaction");
    enqueue_space(&mut tx, "space", 2)
        .await
        .expect("space invalidation");
    sqlx::query("DELETE FROM spaces WHERE id='space'")
        .execute(&mut *tx)
        .await
        .expect("delete space");
    tx.commit().await.expect("commit");

    let pending: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM spaces_retrieval_invalidations WHERE space_id='space'",
    )
    .fetch_one(&pool)
    .await
    .expect("pending invalidation");
    assert_eq!(pending, 1);
}
