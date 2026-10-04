use super::{super::publish_embeddings, fixture::Fixture};
use crate::spaces::knowledge::retrieval::{generation, retired_generations};

const RETENTION_MS: i64 = 7 * 24 * 60 * 60 * 1_000;

#[tokio::test]
async fn incomplete_replacement_retains_predecessor_vectors_and_markers() {
    let fixture = Fixture::new().await;
    publish_embeddings(&fixture.pool, &fixture.index, fixture.input())
        .await
        .unwrap();
    generation::begin_rebuild(&fixture.pool, "space", "replacement")
        .await
        .unwrap();
    retired_generations::drain_with_connection(&fixture.index, &fixture.pool, 2)
        .await
        .unwrap();
    assert_eq!(fixture.rows().await, 1);
    assert_eq!(fixture.markers().await, 1);
    let started: Option<i64> = sqlx::query_scalar(
        "SELECT cleanup_started_at FROM spaces_retrieval_generations WHERE id='generation'",
    )
    .fetch_one(&fixture.pool)
    .await
    .unwrap();
    assert_eq!(started, None);
}

#[tokio::test]
async fn reclamation_preserves_active_vectors_and_canonical_chunks_until_history_expires() {
    let fixture = Fixture::new().await;
    publish_embeddings(&fixture.pool, &fixture.index, fixture.input())
        .await
        .unwrap();
    let replacement = generation::begin_rebuild(&fixture.pool, "space", "replacement")
        .await
        .unwrap();
    let chunks: Vec<(String, String)> = sqlx::query_as(
        "SELECT id,source_revision FROM spaces_current_knowledge_chunks WHERE space_id='space'",
    )
    .fetch_all(&fixture.pool)
    .await
    .unwrap();
    let inputs = chunks
        .into_iter()
        .map(|(chunk_id, source_revision)| {
            let mut input = fixture.input().remove(0);
            input.generation_id.clone_from(&replacement.id);
            input
                .embedding_spec_id
                .clone_from(&replacement.embedding_spec_id);
            input.chunk_id = chunk_id;
            input.source_revision = source_revision;
            input
        })
        .collect();
    publish_embeddings(&fixture.pool, &fixture.index, inputs)
        .await
        .unwrap();
    generation::activate(&fixture.pool, "space", &replacement)
        .await
        .unwrap();
    assert_eq!(fixture.rows().await, 3);
    retired_generations::drain_with_connection(&fixture.index, &fixture.pool, 2)
        .await
        .unwrap();
    assert_eq!(fixture.rows().await, 2);
    assert_eq!(fixture.markers().await, 2);
    let started: i64 = sqlx::query_scalar(
        "SELECT cleanup_started_at FROM spaces_retrieval_generations WHERE id='generation'",
    )
    .fetch_one(&fixture.pool)
    .await
    .unwrap();
    retired_generations::drain_with_connection(
        &fixture.index,
        &fixture.pool,
        started + RETENTION_MS - 1,
    )
    .await
    .unwrap();
    let retained: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM spaces_retrieval_generations WHERE id='generation'",
    )
    .fetch_one(&fixture.pool)
    .await
    .unwrap();
    assert_eq!(retained, 1);
    retired_generations::drain_with_connection(
        &fixture.index,
        &fixture.pool,
        started + RETENTION_MS,
    )
    .await
    .unwrap();
    assert_eq!(fixture.rows().await, 2);
    assert_eq!(fixture.markers().await, 2);
    let chunks: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces_knowledge_chunks")
        .fetch_one(&fixture.pool)
        .await
        .unwrap();
    assert_eq!(chunks, 2);
    assert!(
        publish_embeddings(&fixture.pool, &fixture.index, fixture.input())
            .await
            .is_err()
    );
    let active = generation::active(&fixture.pool, "space", "replacement")
        .await
        .unwrap();
    assert_eq!(active.id, replacement.id);
}
