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
