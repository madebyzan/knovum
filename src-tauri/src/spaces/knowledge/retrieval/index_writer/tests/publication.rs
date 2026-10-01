use std::time::Duration;

use super::super::{into_source, publish_embeddings};
use super::fixture::Fixture;
use crate::spaces::{
    db,
    knowledge::retrieval::{invalidation_outbox, record_loader},
};

#[tokio::test]
async fn publication_drains_old_invalidations_before_writing_new_vectors() {
    let fixture = Fixture::new().await;
    publish_embeddings(&fixture.pool, &fixture.index, fixture.input())
        .await
        .expect("initial publication");
    let mut tx = db::begin_write(&fixture.pool).await.expect("edit");
    invalidation_outbox::enqueue_nodes(&mut tx, "space", &["node".into()], 2)
        .await
        .expect("enqueue");
    tx.commit().await.expect("edit commit");
    publish_embeddings(&fixture.pool, &fixture.index, fixture.input())
        .await
        .expect("fresh publication");
    invalidation_outbox::drain_with_connection(&fixture.index, &fixture.pool)
        .await
        .expect("late drain");
    assert_eq!(fixture.rows().await, 1);
    assert_eq!(fixture.markers().await, 1);
}

#[tokio::test]
async fn invalidation_removes_markers_together_with_vectors() {
    let fixture = Fixture::new().await;
    publish_embeddings(&fixture.pool, &fixture.index, fixture.input())
        .await
        .expect("publication");
    let mut tx = db::begin_write(&fixture.pool).await.expect("edit");
    invalidation_outbox::enqueue_space(&mut tx, "space", 2)
        .await
        .expect("enqueue");
    tx.commit().await.expect("edit commit");
    invalidation_outbox::drain_with_connection(&fixture.index, &fixture.pool)
        .await
        .expect("drain");
    assert_eq!(fixture.rows().await, 0);
    assert_eq!(fixture.markers().await, 0);
}

#[tokio::test]
async fn stale_embedding_cannot_replace_current_vectors() {
    let fixture = Fixture::new().await;
    publish_embeddings(&fixture.pool, &fixture.index, fixture.input())
        .await
        .expect("publication");
    sqlx::query(
        "UPDATE spaces_nodes SET data=json_set(data,'$.sourceRevision','r2') WHERE id='node'",
    )
    .execute(&fixture.pool)
    .await
    .expect("replace source");
    let error = publish_embeddings(&fixture.pool, &fixture.index, fixture.input())
        .await
        .expect_err("stale input");
    assert!(error.contains("no longer has source revision"), "{error}");
    assert_eq!(fixture.rows().await, 1);
}

#[tokio::test]
async fn chunk_deletion_waits_until_index_publication_commits() {
    let fixture = Fixture::new().await;
    let mut publication = db::begin_write(&fixture.pool).await.expect("publication");
    let records = record_loader::load_records(
        &mut publication,
        fixture.input().into_iter().map(into_source).collect(),
    )
    .await
    .expect("validate current chunk");
    let mut deletion = fixture.pool.acquire().await.expect("concurrent connection");
    let delete =
        sqlx::query("DELETE FROM spaces_knowledge_chunks WHERE id='c1'").execute(&mut *deletion);
    tokio::pin!(delete);
    assert!(
        tokio::time::timeout(Duration::from_millis(100), &mut delete)
            .await
            .is_err()
    );
    record_loader::write_index_markers(&mut publication, &records)
        .await
        .expect("marker remains valid");
    publication.commit().await.expect("publication commit");
    delete.await.expect("delete after commit");
    assert_eq!(fixture.markers().await, 0);
}

#[tokio::test]
async fn vector_write_failure_does_not_publish_markers() {
    let fixture = Fixture::new().await;
    publish_embeddings(&fixture.pool, &fixture.index, fixture.input())
        .await
        .expect("publication");
    let mut incompatible = fixture.input();
    incompatible[0].dimensions = 3;
    incompatible[0].vector.push(0.3);
    assert!(
        publish_embeddings(&fixture.pool, &fixture.index, incompatible)
            .await
            .is_err()
    );
    assert_eq!(fixture.rows().await, 1);
    assert_eq!(fixture.markers().await, 1);
    sqlx::query("DELETE FROM spaces_chunk_embeddings")
        .execute(&fixture.pool)
        .await
        .expect("remove markers");
    let mut incompatible = fixture.input();
    incompatible[0].dimensions = 3;
    incompatible[0].vector.push(0.3);
    assert!(
        publish_embeddings(&fixture.pool, &fixture.index, incompatible)
            .await
            .is_err()
    );
    assert_eq!(fixture.rows().await, 1);
    assert_eq!(fixture.markers().await, 0);
}
