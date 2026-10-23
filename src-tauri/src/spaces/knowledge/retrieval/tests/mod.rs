use super::hybrid::reciprocal_rank_fusion;
use super::{generation, index_writer::tests::fixture::Fixture};

fn matched(id: &str, score: f64) -> serde_json::Value {
    serde_json::json!({ "chunkId": id, "score": score })
}

#[test]
fn reciprocal_rank_fusion_deduplicates_chunks() {
    let values = reciprocal_rank_fusion(
        vec![matched("a", 7.0), matched("b", 4.0)],
        vec![matched("b", 8.0), matched("a", 3.0), matched("c", 1.0)],
    );
    let ids = values
        .iter()
        .filter_map(|value| value["chunkId"].as_str())
        .collect::<Vec<_>>();
    assert_eq!(ids.len(), 3);
    assert!(ids.contains(&"a") && ids.contains(&"b") && ids.contains(&"c"));
}

#[test]
fn reciprocal_rank_fusion_keeps_both_score_payloads() {
    let values = reciprocal_rank_fusion(vec![matched("a", 7.0)], vec![matched("a", 3.0)]);
    assert_eq!(values.len(), 1);
    assert_eq!(values[0]["lexicalScore"], 7.0);
    assert_eq!(values[0]["semanticScore"], 3.0);
}

#[tokio::test]
async fn a_rebuild_keeps_the_live_index_until_the_new_generation_is_complete() {
    let fixture = Fixture::new().await;
    sqlx::query(
        "UPDATE spaces_retrieval_generations SET status='active',activated_at=1 WHERE id='generation'",
    )
    .execute(&fixture.pool)
    .await
    .unwrap();

    let rebuilt = generation::begin_rebuild(&fixture.pool, "space", "spec")
        .await
        .unwrap();
    assert_ne!(rebuilt.id, "generation");
    // Re-encoding every chunk of a Space takes as long as it takes. Searches keep answering from
    // the generation that is still complete for the whole build.
    assert_eq!(
        generation::active(&fixture.pool, "space", "spec")
            .await
            .unwrap()
            .id,
        "generation"
    );

    sqlx::query(
        "INSERT INTO spaces_chunk_embeddings(\
           generation_id,chunk_id,source_revision,embedding_spec_id,dimensions,updated_at\
         ) VALUES($1,'c1','r1','spec',2,1)",
    )
    .bind(&rebuilt.id)
    .execute(&fixture.pool)
    .await
    .unwrap();
    generation::activate(&fixture.pool, "space", &rebuilt)
        .await
        .unwrap();

    assert_eq!(
        generation::active(&fixture.pool, "space", "spec")
            .await
            .unwrap()
            .id,
        rebuilt.id
    );
    let replaced: String =
        sqlx::query_scalar("SELECT status FROM spaces_retrieval_generations WHERE id='generation'")
            .fetch_one(&fixture.pool)
            .await
            .unwrap();
    assert_eq!(replaced, "retired");
    fixture.pool.close().await;
}

#[tokio::test]
async fn distinguishes_unprepared_generations_from_database_failures() {
    let fixture = Fixture::new().await;
    assert!(generation::find_active(&fixture.pool, "space", "spec")
        .await
        .unwrap()
        .is_none());
    assert_eq!(
        generation::active(&fixture.pool, "space", "spec")
            .await
            .unwrap_err(),
        generation::REBUILD_REQUIRED
    );

    sqlx::query("UPDATE spaces_retrieval_generations SET status='active',activated_at=1 WHERE id='generation'")
        .execute(&fixture.pool).await.unwrap();
    assert_eq!(
        generation::find_active(&fixture.pool, "space", "spec")
            .await
            .unwrap()
            .unwrap()
            .id,
        "generation"
    );
    assert!(
        generation::find_active(&fixture.pool, "space", "different-spec")
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(
        generation::active(&fixture.pool, "space", "different-spec")
            .await
            .unwrap_err(),
        generation::REBUILD_REQUIRED
    );

    fixture.pool.close().await;
    assert!(generation::find_active(&fixture.pool, "space", "spec")
        .await
        .is_err());
}
